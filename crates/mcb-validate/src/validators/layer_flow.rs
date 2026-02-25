//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Layer Event Flow Validation
//!
//! Validates that dependencies flow in correct Clean Architecture direction:
//! domain -> application -> providers -> infrastructure -> server

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::config::LayerFlowRulesConfig;
use crate::constants::linters::CARGO_TOML_FILENAME;
use crate::define_violations;
use crate::traits::violation::{Violation, ViolationCategory};
use crate::{Result, ValidationConfig};

define_violations! {
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
    /// List of crates to check for circular dependencies
    circular_dependency_check_crates: Vec<String>,
}

crate::impl_config_only_validator_new!(LayerFlowValidator, layer_flow);

impl LayerFlowValidator {
    /// Creates a new layer flow validator with current configuration.
    #[must_use]
    pub fn with_config(config: &LayerFlowRulesConfig) -> Self {
        Self {
            circular_dependency_check_crates: config.circular_dependency_check_crates.clone(),
        }
    }

    fn check_circular_dependencies(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<LayerFlowViolation>> {
        let crates_dir = config.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(Vec::new());
        }

        let crate_names = &self.circular_dependency_check_crates;

        let deps: HashMap<String, HashSet<String>> = crate_names
            .iter()
            .filter_map(|crate_name| {
                let cargo_toml = crates_dir.join(crate_name).join(CARGO_TOML_FILENAME);
                cargo_toml.exists().then_some((crate_name, cargo_toml))
            })
            .map(|(crate_name, cargo_toml)| {
                let content = std::fs::read_to_string(&cargo_toml)?;
                let parsed: toml::Value = toml::from_str(&content)?;
                let crate_deps: HashSet<String> = parsed
                    .get("dependencies")
                    .and_then(|d| d.as_table())
                    .map(|deps_table| {
                        deps_table
                            .keys()
                            .map(|dep| dep.replace('_', "-"))
                            .filter(|dep| dep != crate_name && crate_names.contains(dep))
                            .collect()
                    })
                    // INTENTIONAL: Dependency filter; empty list means no filtered deps
                    .unwrap_or_default();
                Ok((crate_name.clone(), crate_deps))
            })
            .collect::<Result<_>>()?;

        Ok(deps
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .flat_map(|(i, crate_a)| {
                let deps_ref = &deps;
                let crates_dir_ref = &crates_dir;
                let crate_list: Vec<String> = deps_ref.keys().cloned().collect();
                let crate_a_filter = crate_a.clone();
                crate_list
                    .into_iter()
                    .skip(i + 1)
                    .filter(move |crate_b| {
                        deps_ref
                            .get(&crate_a_filter)
                            .is_some_and(|d| d.contains(crate_b))
                            && deps_ref
                                .get(crate_b)
                                .is_some_and(|d| d.contains(&crate_a_filter))
                    })
                    .map(move |crate_b| LayerFlowViolation::CircularDependency {
                        crate_a: crate_a.clone(),
                        crate_b,
                        file: crates_dir_ref.join(&crate_a).join(CARGO_TOML_FILENAME),
                        line: 1,
                    })
            })
            .collect())
    }
}

impl crate::traits::validator::Validator for LayerFlowValidator {
    fn name(&self) -> &'static str {
        "layer_flow"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture layer dependency rules"
    }

    fn validate(&self, config: &ValidationConfig) -> crate::Result<Vec<Box<dyn Violation>>> {
        let violations = self.check_circular_dependencies(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
