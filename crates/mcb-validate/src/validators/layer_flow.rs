//! Layer Event Flow Validation
//!
//! Validates that dependencies flow in correct Clean Architecture direction:
//! domain -> application -> providers -> infrastructure -> server

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use regex::Regex;

use crate::config::LayerFlowRulesConfig;
use crate::scan::for_each_rs_under_root;
use crate::traits::violation::{Violation, ViolationCategory};
use crate::{Result, ValidationConfig};

crate::define_violations! {
    ViolationCategory::Architecture,
    pub enum LayerFlowViolation {
        /// Dependency detected that violates the allowed layer flow.
        #[violation(
            id = "LAYER001",
            severity = Error,
            message = "CA: Forbidden import in {source_crate}: {import_path} (imports {target_crate}) at {file}:{line}",
            suggestion = "Remove {target_crate} from {source_crate} - violates CA"
        )]
        ForbiddenDependency {
            source_crate: String,
            target_crate: String,
            import_path: String,
            file: PathBuf,
            line: usize,
        },
        /// Circular dependency detected between crates.
        #[violation(
            id = "LAYER002",
            severity = Error,
            message = "CA: Circular dependency: {crate_a} <-> {crate_b} at {file}:{line}",
            suggestion = "Extract shared types to the domain crate"
        )]
        CircularDependency {
            crate_a: String,
            crate_b: String,
            file: PathBuf,
            line: usize,
        },
        /// Domain layer importing external crates (should be pure).
        #[violation(
            id = "LAYER003",
            severity = Warning,
            message = "CA: Domain {crate_name} imports external: {external_crate} at {file}:{line}",
            suggestion = "Domain should only use std/serde/thiserror"
        )]
        DomainExternalDependency {
            crate_name: String,
            external_crate: String,
            file: PathBuf,
            line: usize,
        },
    }
}

/// Layer Flow Validator
pub struct LayerFlowValidator {
    /// Forbidden dependency mappings: source_crate -> forbidden_target_crates
    forbidden_dependencies: HashMap<String, HashSet<String>>,
    /// List of crates to check for circular dependencies
    circular_dependency_check_crates: Vec<String>,
}

impl LayerFlowValidator {
    /// Creates a new layer flow validator, loading rules from configuration.
    pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        let file_config = crate::config::FileConfig::load(workspace_root);
        Self::with_config(&file_config.rules.layer_flow)
    }

    /// Creates a new layer flow validator with current configuration.
    pub fn with_config(config: &LayerFlowRulesConfig) -> Self {
        let mut forbidden_dependencies = HashMap::new();
        for (k, v) in &config.forbidden_dependencies {
            forbidden_dependencies.insert(k.clone(), v.iter().cloned().collect());
        }

        Self {
            forbidden_dependencies,
            circular_dependency_check_crates: config.circular_dependency_check_crates.clone(),
        }
    }

    /// Validates the layer flow constraints for the given configuration.
    pub fn validate(&self, config: &ValidationConfig) -> Result<Vec<LayerFlowViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.check_forbidden_imports(config)?);
        violations.extend(self.check_circular_dependencies(config)?);
        Ok(violations)
    }

    fn check_forbidden_imports(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<LayerFlowViolation>> {
        let mut violations = Vec::new();
        let crates_dir = config.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(violations);
        }

        let import_pattern = Regex::new(r"use\s+([\w][\w\d_]*)").expect("Invalid regex");

        for crate_name in self.forbidden_dependencies.keys() {
            let crate_src_dir = crates_dir.join(crate_name).join("src");
            if !crate_src_dir.exists() {
                continue;
            }

            let forbidden_deps = &self.forbidden_dependencies[crate_name];
            let crate_name_underscored = crate_name.replace('-', "_");

            for_each_rs_under_root(config, &crate_src_dir, |path| {
                let content = std::fs::read_to_string(path)?;
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                        continue;
                    }

                    for captures in import_pattern.captures_iter(line) {
                        let imported_crate = captures.get(1).map_or("", |m| m.as_str());
                        let imported_crate_dashed = imported_crate.replace('_', "-");
                        if imported_crate == crate_name_underscored {
                            continue;
                        }
                        if forbidden_deps.contains(imported_crate_dashed.as_str()) {
                            violations.push(LayerFlowViolation::ForbiddenDependency {
                                source_crate: crate_name.to_string(),
                                target_crate: imported_crate_dashed,
                                import_path: line.trim().to_string(),
                                file: path.to_path_buf(),
                                line: line_num + 1,
                            });
                        }
                    }
                }

                Ok(())
            })?;
        }
        Ok(violations)
    }

    fn check_circular_dependencies(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<LayerFlowViolation>> {
        let mut violations = Vec::new();
        let crates_dir = config.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(violations);
        }

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
        let crate_names = &self.circular_dependency_check_crates;

        for crate_name in crate_names {
            let cargo_toml = crates_dir.join(crate_name).join("Cargo.toml");
            if !cargo_toml.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&cargo_toml)?;
            let mut crate_deps = HashSet::new();
            let mut in_dev_deps = false;
            let mut in_build_deps = false;

            for line in content.lines() {
                let trimmed = line.trim();

                // Skip comments - they're not actual dependencies
                if trimmed.starts_with('#') {
                    continue;
                }

                // Track section changes
                if trimmed.starts_with('[') {
                    in_dev_deps = trimmed.contains("dev-dependencies");
                    in_build_deps = trimmed.contains("build-dependencies");
                }

                // Skip dev-dependencies and build-dependencies (not part of runtime graph)
                if in_dev_deps || in_build_deps {
                    continue;
                }

                // Only match actual dependency declarations, not any mention of crate name
                // Look for patterns like: crate-name = { path = ... } or crate-name.path = ...
                for dep_crate in crate_names {
                    if dep_crate != crate_name
                        && (trimmed.starts_with(dep_crate)
                            || trimmed.contains(&format!("\"{dep_crate}\"")))
                    {
                        crate_deps.insert(dep_crate.to_string());
                    }
                }
            }
            deps.insert(crate_name.to_string(), crate_deps);
        }

        let crate_list: Vec<_> = deps.keys().cloned().collect();
        for (i, crate_a) in crate_list.iter().enumerate() {
            for crate_b in crate_list.iter().skip(i + 1) {
                let a_deps_b = deps.get(crate_a).is_some_and(|d| d.contains(crate_b));
                let b_deps_a = deps.get(crate_b).is_some_and(|d| d.contains(crate_a));
                if a_deps_b && b_deps_a {
                    violations.push(LayerFlowViolation::CircularDependency {
                        crate_a: crate_a.clone(),
                        crate_b: crate_b.clone(),
                        file: crates_dir.join(crate_a).join("Cargo.toml"),
                        line: 1,
                    });
                }
            }
        }
        Ok(violations)
    }
}

impl crate::traits::validator::Validator for LayerFlowValidator {
    fn name(&self) -> &'static str {
        "layer_flow"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture layer dependency rules"
    }

    fn validate(&self, config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
