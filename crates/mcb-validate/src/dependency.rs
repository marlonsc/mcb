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
use walkdir::WalkDir;

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

/// Dependency violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyViolation {
    /// Forbidden dependency in Cargo.toml
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
    CircularDependency {
        /// The sequence of crates forming the dependency cycle.
        cycle: Vec<String>,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Admin surface imports repository ports outside approved composition roots.
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

impl DependencyViolation {
    /// Returns the severity level of this violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
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

impl Violation for DependencyViolation {
    fn id(&self) -> &str {
        match self {
            Self::ForbiddenCargoDepedency { .. } => "DEP001",
            Self::ForbiddenUseStatement { .. } => "DEP002",
            Self::CircularDependency { .. } => "DEP003",
            Self::AdminBypassImport { .. } => "DEP004",
            Self::CliBypassPath { .. } => "DEP005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::ForbiddenCargoDepedency { severity, .. }
            | Self::ForbiddenUseStatement { severity, .. }
            | Self::CircularDependency { severity, .. }
            | Self::AdminBypassImport { severity, .. }
            | Self::CliBypassPath { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::ForbiddenCargoDepedency { location, .. } => Some(location),
            Self::ForbiddenUseStatement { file, .. } => Some(file),
            Self::CircularDependency { .. } => None,
            Self::AdminBypassImport { file, .. } | Self::CliBypassPath { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::ForbiddenUseStatement { line, .. }
            | Self::AdminBypassImport { line, .. }
            | Self::CliBypassPath { line, .. } => Some(*line),
            _ => None,
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::ForbiddenCargoDepedency {
                crate_name,
                forbidden_dep,
                ..
            } => Some(format!(
                "Remove {forbidden_dep} from {crate_name}/Cargo.toml"
            )),
            Self::ForbiddenUseStatement { forbidden_dep, .. } => {
                Some(format!("Access {forbidden_dep} through allowed layer"))
            }
            Self::CircularDependency { .. } => {
                Some("Extract shared types to the domain crate".to_string())
            }
            Self::AdminBypassImport { .. } => Some(
                "Route admin operations through ToolHandlers/UnifiedExecution and keep repository imports in approved composition roots only".to_string(),
            ),
            Self::CliBypassPath { .. } => Some(
                "Route CLI business commands through the unified execution path instead of direct mcb_validate calls".to_string(),
            ),
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

        for entry in WalkDir::new(scan_root)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let rel = entry
                .path()
                .strip_prefix(&self.config.workspace_root)
                .unwrap_or(entry.path());
            if !should_check_file(rel) {
                continue;
            }

            let content = std::fs::read_to_string(entry.path())?;
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if line.contains(pattern) {
                    out.push(make_violation(
                        entry.path().to_path_buf(),
                        line_num + 1,
                        trimmed.to_string(),
                    ));
                }
            }
        }

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

            for entry in WalkDir::new(&crate_src)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

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
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                context: line.trim().to_string(),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
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
