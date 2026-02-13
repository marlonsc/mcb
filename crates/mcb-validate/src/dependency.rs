//! Dependency Graph Validation
//!
//! Validates Clean Architecture layer boundaries:
//! - domain: No internal dependencies (pure domain entities)
//! - application: Only domain (use cases and ports)
//! - providers: domain and application (adapter implementations)
//! - infrastructure: domain, application, and providers (DI composition root)
//! - server: domain, application, and infrastructure (transport layer)
//! - mcb: All crates (facade that re-exports entire public API)

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::scan::for_each_rs_under_root;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

define_violations! {
    no_display,
    ViolationCategory::Architecture,
    pub enum DependencyViolation {
        /// Forbidden dependency in Cargo.toml
        violation(
            id = "DEP001",
            severity = Error,
            suggestion = "Remove {forbidden_dep} from {crate_name}/Cargo.toml"
        )
        ForbiddenCargoDepedency {
            /// Name of the crate that contains the forbidden dependency.
            crate_name: String,
            /// Name of the crate that is forbidden as a dependency.
            forbidden_dep: String,
            /// Path to the Cargo.toml file containing the violation.
            location: PathBuf,
            /// Severity level of the violation.
            severity: Severity,
        },
        /// Forbidden use statement in source code
        violation(
            id = "DEP002",
            severity = Error,
            suggestion = "Access {forbidden_dep} through allowed layer"
        )
        ForbiddenUseStatement {
            /// Name of the crate where the violation was found.
            crate_name: String,
            /// Name of the crate whose items are being incorrectly imported.
            forbidden_dep: String,
            /// Path to the source file containing the violation.
            file: PathBuf,
            /// Line number where the forbidden `use` statement occurs.
            line: usize,
            /// The content of the line containing the violation.
            context: String,
            /// Severity level of the violation.
            severity: Severity,
        },
        /// Circular dependency detected
        violation(
            id = "DEP003",
            severity = Error,
            suggestion = "Extract shared types to the domain crate"
        )
        CircularDependency {
            /// The sequence of crates forming the dependency cycle.
            cycle: Vec<String>,
            /// Severity level of the violation.
            severity: Severity,
        },
        /// Admin surface imports repository ports outside approved composition roots.
        violation(
            id = "DEP004",
            severity = Error,
            suggestion = "Route admin operations through ToolHandlers/UnifiedExecution and keep repository imports in approved composition roots only"
        )
        AdminBypassImport {
            /// Source file that introduced the bypass import.
            file: PathBuf,
            /// Line where bypass import appears.
            line: usize,
            /// Offending source line.
            context: String,
            /// Severity level of the violation.
            severity: Severity,
        },
        /// CLI code bypasses unified execution by calling validate crate directly.
        violation(
            id = "DEP005",
            severity = Error,
            suggestion = "Route CLI business commands through the unified execution path instead of direct mcb_validate calls"
        )
        CliBypassPath {
            /// Source file that introduced direct validation execution.
            file: PathBuf,
            /// Line where bypass call/import appears.
            line: usize,
            /// Offending source line.
            context: String,
            /// Severity level of the violation.
            severity: Severity,
        },
    }
}

impl std::fmt::Display for DependencyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForbiddenCargoDepedency {
                crate_name,
                forbidden_dep,
                location,
                ..
            } => {
                write!(
                    f,
                    "Forbidden dependency: {} depends on {} (in {})",
                    crate_name,
                    forbidden_dep,
                    location.display()
                )
            }
            Self::ForbiddenUseStatement {
                crate_name,
                forbidden_dep,
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "Forbidden use: {} uses {} at {}:{} - {}",
                    crate_name,
                    forbidden_dep,
                    file.display(),
                    line,
                    context
                )
            }
            Self::CircularDependency { cycle, .. } => {
                write!(f, "Circular dependency: {}", cycle.join(" -> "))
            }
            Self::AdminBypassImport {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "Admin bypass import: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::CliBypassPath {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "CLI bypass path: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
        }
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

        if allowed_deps.is_empty() {
            panic!(
                "DependencyValidator: No allowed dependencies found in YAML rules CA001-CA016. Configuration is required in crates/mcb-validate/rules/."
            );
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

    /// Validate anti-bypass boundaries for admin and CLI surfaces.
    pub fn validate_bypass_boundaries(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();

        let admin_allowed_import_roots = [
            "crates/mcb-server/src/admin/crud_adapter.rs",
            "crates/mcb-server/src/admin/handlers.rs",
        ];
        let cli_allowed_direct_validate = ["crates/mcb/src/cli/validate.rs"];

        let admin_root = self
            .config
            .workspace_root
            .join("crates")
            .join("mcb-server")
            .join("src")
            .join("admin");
        self.scan_bypass_patterns(
            &admin_root,
            |rel| {
                !admin_allowed_import_roots
                    .iter()
                    .any(|allowed| rel == Path::new(allowed))
            },
            "mcb_domain::ports::repositories",
            |file, line, context| DependencyViolation::AdminBypassImport {
                file,
                line,
                context,
                severity: Severity::Error,
            },
            &mut violations,
        )?;

        let cli_root = self
            .config
            .workspace_root
            .join("crates")
            .join("mcb")
            .join("src")
            .join("cli");
        self.scan_bypass_patterns(
            &cli_root,
            |rel| {
                !cli_allowed_direct_validate
                    .iter()
                    .any(|allowed| rel == Path::new(allowed))
            },
            "mcb_validate::",
            |file, line, context| DependencyViolation::CliBypassPath {
                file,
                line,
                context,
                severity: Severity::Error,
            },
            &mut violations,
        )?;

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

        for_each_rs_under_root(&self.config, scan_root, |path| {
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
        let use_pattern = Regex::new(r"use\s+(mcb_[a-z_]+)").unwrap();

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

            for_each_rs_under_root(&self.config, &crate_src, |path| {
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
                    cycle,
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

impl_validator!(
    DependencyValidator,
    "dependency",
    "Validates Clean Architecture layer dependencies"
);
