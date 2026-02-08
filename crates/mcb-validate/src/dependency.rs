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
use std::path::PathBuf;

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
}

impl DependencyViolation {
    /// Returns the severity level of this violation.
    pub fn severity(&self) -> Severity {
        match self {
            Self::ForbiddenCargoDepedency { severity, .. }
            | Self::ForbiddenUseStatement { severity, .. }
            | Self::CircularDependency { severity, .. } => *severity,
        }
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
        }
    }
}

impl Violation for DependencyViolation {
    fn id(&self) -> &str {
        match self {
            Self::ForbiddenCargoDepedency { .. } => "DEP001",
            Self::ForbiddenUseStatement { .. } => "DEP002",
            Self::CircularDependency { .. } => "DEP003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::ForbiddenCargoDepedency { severity, .. }
            | Self::ForbiddenUseStatement { severity, .. }
            | Self::CircularDependency { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::ForbiddenCargoDepedency { location, .. } => Some(location),
            Self::ForbiddenUseStatement { file, .. } => Some(file),
            Self::CircularDependency { .. } => None,
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::ForbiddenUseStatement { line, .. } => Some(*line),
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

        // Load from YAML rules CA001-CA016
        // We look for any CA rules that have crate_name and allowed_dependencies in their config
        for i in 1..=20 {
            // Check both CAXXX and LAYERXXX (legacy mapping)
            let ids = [format!("CA{:03}", i), format!("LAYER{:03}", i)];
            for rule_id in ids {
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
        Ok(violations)
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

impl crate::validator_trait::Validator for DependencyValidator {
    fn name(&self) -> &'static str {
        "dependency"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture layer dependencies"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn create_test_workspace() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Create workspace Cargo.toml
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["crates/mcb-domain", "crates/mcb-infrastructure"]
"#,
        )
        .unwrap();

        // Create mcb-domain crate (no deps)
        let domain_dir = temp.path().join("crates/mcb-domain");
        fs::create_dir_all(domain_dir.join("src")).unwrap();
        fs::write(
            domain_dir.join("Cargo.toml"),
            r#"
[package]
name = "mcb-domain"
version = "0.1.1"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();
        fs::write(domain_dir.join("src/lib.rs"), "pub fn domain() {}").unwrap();

        // Create mcb-infrastructure crate (depends on domain)
        let infra_dir = temp.path().join("crates/mcb-infrastructure");
        fs::create_dir_all(infra_dir.join("src")).unwrap();
        fs::write(
            infra_dir.join("Cargo.toml"),
            r#"
[package]
name = "mcb-infrastructure"
version = "0.1.1"

[dependencies]
mcb-domain = { path = "../mcb-domain" }
"#,
        )
        .unwrap();
        fs::write(
            infra_dir.join("src/lib.rs"),
            "use mcb_domain::domain;\npub fn infra() { domain(); }",
        )
        .unwrap();

        temp
    }

    #[test]
    fn test_valid_dependencies() {
        let temp = create_test_workspace();
        let validator = DependencyValidator::new(temp.path());

        let violations = validator.validate_cargo_dependencies().unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations, got: {violations:?}"
        );
    }

    #[test]
    fn test_forbidden_dependency() {
        let temp = TempDir::new().unwrap();

        // Create domain crate that incorrectly depends on infrastructure
        let domain_dir = temp.path().join("crates/mcb-domain");
        fs::create_dir_all(domain_dir.join("src")).unwrap();
        fs::write(
            domain_dir.join("Cargo.toml"),
            r#"
[package]
name = "mcb-domain"
version = "0.1.1"

[dependencies]
mcb-infrastructure = { path = "../mcb-infrastructure" }
"#,
        )
        .unwrap();
        fs::write(domain_dir.join("src/lib.rs"), "").unwrap();

        let validator = DependencyValidator::new(temp.path());
        let violations = validator.validate_cargo_dependencies().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            DependencyViolation::ForbiddenCargoDepedency {
                crate_name,
                forbidden_dep,
                ..
            } => {
                assert_eq!(crate_name, "mcb-domain");
                assert_eq!(forbidden_dep, "mcb-infrastructure");
            }
            _ => panic!("Expected ForbiddenCargoDependency"),
        }
    }

    #[test]
    fn test_forbidden_use_statement() {
        let temp = TempDir::new().unwrap();

        // Create domain crate with forbidden use statement
        let domain_dir = temp.path().join("crates/mcb-domain");
        fs::create_dir_all(domain_dir.join("src")).unwrap();
        fs::write(
            domain_dir.join("Cargo.toml"),
            r#"
[package]
name = "mcb-domain"
version = "0.1.1"
"#,
        )
        .unwrap();
        fs::write(
            domain_dir.join("src/lib.rs"),
            "use mcb_infrastructure::something;",
        )
        .unwrap();

        let validator = DependencyValidator::new(temp.path());
        let violations = validator.validate_use_statements().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            DependencyViolation::ForbiddenUseStatement {
                crate_name,
                forbidden_dep,
                ..
            } => {
                assert_eq!(crate_name, "mcb-domain");
                assert_eq!(forbidden_dep, "mcb-infrastructure");
            }
            _ => panic!("Expected ForbiddenUseStatement"),
        }
    }
}
