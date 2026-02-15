//! Dependency Graph Validation
//!
//! Validates Clean Architecture layer boundaries:
//! - domain: No internal dependencies (pure domain entities)
//! - application: Only domain (use cases and ports)
//! - providers: domain and application (adapter implementations)
//! - infrastructure: domain, application, and providers (DI composition root)
//! - server: domain, application, and infrastructure (transport layer)
//! - mcb: All crates (facade that re-exports entire public API)

use crate::filters::LanguageId;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::pattern_registry::compile_regex;
use crate::scan::for_each_file_under_root;
use crate::traits::violation::ViolationCategory;
use crate::{Result, Severity, ValidationConfig};

/// Wrapper for dependency cycle to provide custom formatting.
#[derive(Clone, Serialize, Deserialize)]
pub struct DependencyCycle(pub Vec<String>);

impl std::fmt::Debug for DependencyCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" -> "))
    }
}

crate::define_violations! {
    dynamic_severity,
    ViolationCategory::Architecture,
    pub enum DependencyViolation {
        /// Forbidden dependency in Cargo.toml
        #[violation(
            id = "DEP001",
            severity = Error,
            message = "Forbidden dependency: {crate_name} depends on {forbidden_dep} (in {location})",
            suggestion = "Remove {forbidden_dep} from {crate_name}/Cargo.toml"
        )]
        ForbiddenCargoDepedency {
            crate_name: String,
            forbidden_dep: String,
            location: PathBuf,
            severity: Severity,
        },
        /// Forbidden use statement in source code
        #[violation(
            id = "DEP002",
            severity = Error,
            message = "Forbidden use: {crate_name} uses {forbidden_dep} at {file}:{line} - {context}",
            suggestion = "Access {forbidden_dep} through allowed layer"
        )]
        ForbiddenUseStatement {
            crate_name: String,
            forbidden_dep: String,
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// Circular dependency detected
        #[violation(
            id = "DEP003",
            severity = Error,
            message = "Circular dependency: {cycle}",
            suggestion = "Extract shared types to the domain crate"
        )]
        CircularDependency {
            cycle: DependencyCycle,
            severity: Severity,
        },
        /// Admin surface imports repository ports outside approved composition roots.
        #[violation(
            id = "DEP004",
            severity = Error,
            message = "Admin bypass import: {file}:{line} - {context}",
            suggestion = "Route admin operations through ToolHandlers/UnifiedExecution and keep repository imports in approved composition roots only"
        )]
        AdminBypassImport {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// CLI code bypasses unified execution by calling validate crate directly.
        #[violation(
            id = "DEP005",
            severity = Error,
            message = "CLI bypass path: {file}:{line} - {context}",
            suggestion = "Route CLI business commands through the unified execution path instead of direct mcb_validate calls"
        )]
        CliBypassPath {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
    }
}

/// Validates Clean Architecture dependency rules across crates.
///
/// Ensures that crates only depend on allowed layers according to Clean Architecture principles.
/// Validates both Cargo.toml dependencies and use statements in source code.
pub struct DependencyValidator {
    config: ValidationConfig,
    allowed_deps: HashMap<String, HashSet<String>>,
}

impl DependencyValidator {
    /// Create a new dependency validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        use crate::pattern_registry::PATTERNS;
        let mut allowed_deps = HashMap::new();

        for i in 1..=20 {
            let rule_id = format!("CA{:03}", i);
            if let Some(config_val) = PATTERNS.get_config(&rule_id)
                && let Some(crate_name) = config_val.get("crate_name").and_then(|v| v.as_str())
            {
                let deps: HashSet<String> = PATTERNS
                    .get_config_list(&rule_id, "allowed_dependencies")
                    .into_iter()
                    .collect();
                allowed_deps.insert(crate_name.to_string(), deps);
            }
        }

        Self {
            config,
            allowed_deps,
        }
    }

    /// Run all dependency validations
    pub fn validate_all(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_cargo_dependencies()?);
        violations.extend(self.validate_use_statements()?);
        violations.extend(self.detect_circular_dependencies()?);
        violations.extend(self.validate_bypass_boundaries()?);
        Ok(violations)
    }

    /// Validate anti-bypass boundaries from config.
    pub fn validate_bypass_boundaries(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();
        let file_config = crate::config::FileConfig::load(&self.config.workspace_root);

        for boundary in &file_config.rules.dependency.bypass_boundaries {
            let scan_root = self.config.workspace_root.join(&boundary.scan_root);
            let allowed: Vec<&str> = boundary.allowed_files.iter().map(|s| s.as_str()).collect();
            let violation_id = boundary.violation_id.clone();
            let pattern = boundary.pattern.clone();

            self.scan_bypass_patterns(
                &scan_root,
                |rel| !allowed.iter().any(|a| rel == Path::new(a)),
                &pattern,
                |file, line, context| match violation_id.as_str() {
                    "DEP004" => DependencyViolation::AdminBypassImport {
                        file,
                        line,
                        context,
                        severity: Severity::Error,
                    },
                    "DEP005" => DependencyViolation::CliBypassPath {
                        file,
                        line,
                        context,
                        severity: Severity::Error,
                    },
                    _ => DependencyViolation::AdminBypassImport {
                        file,
                        line,
                        context,
                        severity: Severity::Error,
                    },
                },
                &mut violations,
            )?;
        }

        Ok(violations)
    }

    fn scan_bypass_patterns<F, G>(
        &self,
        scan_root: &Path,
        should_check_file: F,
        pattern: &str,
        make_violation: G,
        out: &mut Vec<DependencyViolation>,
    ) -> Result<()>
    where
        F: Fn(&Path) -> bool,
        G: Fn(PathBuf, usize, String) -> DependencyViolation,
    {
        if !scan_root.exists() {
            return Ok(());
        }

        for_each_file_under_root(&self.config, scan_root, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            let rel = path
                .strip_prefix(&self.config.workspace_root)
                .unwrap_or(path);
            if !should_check_file(rel) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if line.contains(pattern) {
                    out.push(make_violation(
                        path.to_path_buf(),
                        line_num + 1,
                        trimmed.to_string(),
                    ));
                }
            }

            Ok(())
        })?;

        Ok(())
    }

    /// Validate Cargo.toml dependencies match Clean Architecture rules
    pub fn validate_cargo_dependencies(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();

        for (crate_name, allowed) in &self.allowed_deps {
            let cargo_toml = self
                .config
                .workspace_root
                .join("crates")
                .join(crate_name)
                .join("Cargo.toml");

            if !cargo_toml.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&cargo_toml)?;
            let parsed: toml::Value = toml::from_str(&content)?;

            // Check [dependencies] section
            if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                for dep_name in deps.keys() {
                    if dep_name.starts_with("mcb") && dep_name != crate_name {
                        let dep_crate = dep_name.replace('_', "-");
                        if !allowed.contains(&dep_crate) {
                            violations.push(DependencyViolation::ForbiddenCargoDepedency {
                                crate_name: crate_name.clone(),
                                forbidden_dep: dep_crate,
                                location: cargo_toml.clone(),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }

            // Check [dev-dependencies] section (more lenient - allow test utilities)
            // Dev dependencies are allowed to have more flexibility
        }

        Ok(violations)
    }

    /// Validate no forbidden use statements in source code
    pub fn validate_use_statements(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();
        let use_pattern = compile_regex(r"use\s+(mcb_[a-z_]+)")?;

        for (crate_name, allowed) in &self.allowed_deps {
            let crate_src = self
                .config
                .workspace_root
                .join("crates")
                .join(crate_name)
                .join("src");

            if !crate_src.exists() {
                continue;
            }

            for_each_file_under_root(&self.config, &crate_src, Some(LanguageId::Rust), |entry| {
                let path = &entry.absolute_path;
                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    // Skip comments
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                        continue;
                    }

                    // Skip lines that are likely string literals (contain quotes)
                    if line.contains('"') {
                        continue;
                    }

                    for cap in use_pattern.captures_iter(line) {
                        let used_crate = cap.get(1).map_or("", |m| m.as_str());
                        let used_crate_kebab = used_crate.replace('_', "-");

                        // Skip self-references
                        if used_crate_kebab == *crate_name {
                            continue;
                        }

                        // Check if this dependency is allowed
                        if !allowed.contains(&used_crate_kebab) {
                            violations.push(DependencyViolation::ForbiddenUseStatement {
                                crate_name: crate_name.clone(),
                                forbidden_dep: used_crate_kebab,
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                context: line.trim().to_string(),
                                severity: Severity::Error,
                            });
                        }
                    }
                }

                Ok(())
            })?;
        }

        Ok(violations)
    }

    /// Detect circular dependencies using topological sort
    pub fn detect_circular_dependencies(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        // Build dependency graph from Cargo.toml files
        for crate_name in self.allowed_deps.keys() {
            let cargo_toml = self
                .config
                .workspace_root
                .join("crates")
                .join(crate_name)
                .join("Cargo.toml");

            if !cargo_toml.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&cargo_toml)?;
            let parsed: toml::Value = toml::from_str(&content)?;

            let mut deps = HashSet::new();
            if let Some(dependencies) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                for dep_name in dependencies.keys() {
                    if dep_name.starts_with("mcb") {
                        deps.insert(dep_name.replace('_', "-"));
                    }
                }
            }
            graph.insert(crate_name.clone(), deps);
        }

        // Detect cycles using DFS
        for start in graph.keys() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            if let Some(cycle) = find_cycle_impl(&graph, start, &mut visited, &mut path) {
                violations.push(DependencyViolation::CircularDependency {
                    cycle: DependencyCycle(cycle),
                    severity: Severity::Error,
                });
            }
        }

        Ok(violations)
    }
}

/// Detects cycles in the dependency graph using depth-first search.
///
/// Recursively traverses the dependency graph to find circular dependencies.
/// Returns the cycle path if found, or None if no cycle exists from this node.
///
/// # Arguments
/// * `graph` - The dependency graph mapping crate names to their dependencies
/// * `node` - The current node being visited
/// * `visited` - Set of nodes already fully explored
/// * `path` - Current path being explored (used to detect cycles)
fn find_cycle_impl(
    graph: &HashMap<String, HashSet<String>>,
    node: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if path.contains(&node.to_string()) {
        let cycle_start = path.iter().position(|n| n == node)?;
        let mut cycle: Vec<String> = path[cycle_start..].to_vec();
        cycle.push(node.to_string());
        return Some(cycle);
    }

    if visited.contains(node) {
        return None;
    }

    visited.insert(node.to_string());
    path.push(node.to_string());

    if let Some(deps) = graph.get(node) {
        for dep in deps {
            if let Some(cycle) = find_cycle_impl(graph, dep, visited, path) {
                return Some(cycle);
            }
        }
    }

    path.pop();
    None
}

crate::impl_validator!(
    DependencyValidator,
    "dependency",
    "Validates Clean Architecture layer dependencies"
);
