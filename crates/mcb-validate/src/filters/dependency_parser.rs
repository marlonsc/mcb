//! Cargo.toml Dependency Parser
//!
//! Parses Cargo.toml files to extract declared dependencies for validation.
//! Used to check if libraries used in code are properly declared.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::Result;

/// Information about a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Whether the dependency is declared in Cargo.toml
    pub declared: bool,
    /// Whether the dependency is used in code (filled during analysis)
    pub used_in_code: bool,
    /// Version requirement (if specified)
    pub version: Option<String>,
    /// Features enabled (if any)
    pub features: Vec<String>,
}

/// Dependencies for a single crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateDependencies {
    /// Map of dependency name -> info
    pub deps: HashMap<String, DependencyInfo>,
}

impl CrateDependencies {
    /// Check if a dependency is declared
    pub fn has_dependency(&self, name: &str) -> bool {
        self.deps.contains_key(name)
    }

    /// Get dependency info
    pub fn get_dependency(&self, name: &str) -> Option<&DependencyInfo> {
        self.deps.get(name)
    }

    /// Mark a dependency as used
    pub fn mark_used(&mut self, name: &str) {
        if let Some(dep) = self.deps.get_mut(name) {
            dep.used_in_code = true;
        }
    }
}

/// Dependencies for the entire workspace
#[derive(Debug, Clone)]
pub struct WorkspaceDependencies {
    /// Map of crate directory -> dependencies
    pub deps: HashMap<PathBuf, CrateDependencies>,
}

impl WorkspaceDependencies {
    /// Find dependencies for a specific file's crate
    pub fn find_crate_deps(&self, file_path: &Path) -> Option<&CrateDependencies> {
        // Find the crate directory containing this file
        let mut current = file_path.parent()?;
        loop {
            // Check if this directory contains Cargo.toml
            if current.join("Cargo.toml").exists() {
                return self.deps.get(current);
            }

            // Move up one directory
            current = current.parent()?;
        }
    }

    /// Get all declared dependencies across workspace
    pub fn all_declared_deps(&self) -> HashMap<String, Vec<PathBuf>> {
        let mut result = HashMap::new();

        for (crate_path, crate_deps) in &self.deps {
            for dep_name in crate_deps.deps.keys() {
                result
                    .entry(dep_name.clone())
                    .or_insert_with(Vec::new)
                    .push(crate_path.clone());
            }
        }

        result
    }
}

/// Parser for Cargo.toml dependencies
pub struct CargoDependencyParser {
    workspace_root: PathBuf,
}

impl CargoDependencyParser {
    /// Create a new parser for the given workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Parse dependencies for the entire workspace
    pub fn parse_workspace_deps(&self) -> Result<WorkspaceDependencies> {
        let mut deps = HashMap::new();

        // Find all crate directories (directories with Cargo.toml)
        let crate_dirs = self.find_crate_dirs();

        for crate_dir in crate_dirs {
            let cargo_toml_path = crate_dir.join("Cargo.toml");
            if cargo_toml_path.exists() {
                let crate_deps = self.parse_cargo_toml(&cargo_toml_path)?;
                deps.insert(crate_dir, crate_deps);
            }
        }

        Ok(WorkspaceDependencies { deps })
    }

    /// Find all crate directories in the workspace
    fn find_crate_dirs(&self) -> Vec<PathBuf> {
        let mut crates = Vec::new();

        if self.workspace_root.join("Cargo.toml").exists() {
            crates.push(self.workspace_root.clone());
        }

        for entry in WalkDir::new(&self.workspace_root)
            .max_depth(3)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml")
                && let Some(parent) = path.parent()
                && parent != self.workspace_root
            {
                crates.push(parent.to_path_buf());
            }
        }

        crates
    }

    /// Parse a single Cargo.toml file
    fn parse_cargo_toml(&self, path: &Path) -> Result<CrateDependencies> {
        let content = fs::read_to_string(path)?;
        let value: toml::Value =
            toml::from_str(&content).map_err(|e| crate::ValidationError::Parse {
                file: path.to_path_buf(),
                message: format!("Failed to parse Cargo.toml: {e}"),
            })?;

        let mut deps = HashMap::new();

        if let Some(deps_section) = value.get("dependencies") {
            self.parse_dependency_table(deps_section, &mut deps, false);
        }
        if let Some(dev_deps_section) = value.get("dev-dependencies") {
            self.parse_dependency_table(dev_deps_section, &mut deps, false);
        }
        if let Some(build_deps_section) = value.get("build-dependencies") {
            self.parse_dependency_table(build_deps_section, &mut deps, false);
        }

        Ok(CrateDependencies { deps })
    }

    /// Parse a dependency table (dependencies, dev-dependencies, etc.)
    fn parse_dependency_table(
        &self,
        deps_section: &toml::Value,
        deps: &mut HashMap<String, DependencyInfo>,
        is_optional: bool,
    ) {
        if let Some(table) = deps_section.as_table() {
            for (name, config) in table {
                let info = self.parse_dependency_config(config, is_optional);
                deps.insert(name.clone(), info);
            }
        }
    }

    /// Parse individual dependency configuration
    fn parse_dependency_config(&self, config: &toml::Value, is_optional: bool) -> DependencyInfo {
        match config {
            // Simple version string: serde = "1.0"
            toml::Value::String(version) => DependencyInfo {
                declared: !is_optional,
                used_in_code: false,
                version: Some(version.clone()),
                features: Vec::new(),
            },

            // Table format: serde = { version = "1.0", features = ["derive"] }
            toml::Value::Table(table) => {
                let version = table
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(std::string::ToString::to_string);

                let features = table
                    .get("features")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default();

                let optional = table
                    .get("optional")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false);

                DependencyInfo {
                    declared: !is_optional && !optional,
                    used_in_code: false,
                    version,
                    features,
                }
            }

            _ => DependencyInfo {
                declared: !is_optional,
                used_in_code: false,
                version: None,
                features: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_simple_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#;

        fs::write(&cargo_toml, content).unwrap();

        let parser = CargoDependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_cargo_toml(&cargo_toml).unwrap();

        assert!(deps.has_dependency("serde"));
        assert!(deps.has_dependency("tokio"));

        let serde_info = deps.get_dependency("serde").unwrap();
        assert!(serde_info.declared);
        assert_eq!(serde_info.version, Some("1.0".to_string()));
        assert!(serde_info.features.is_empty());

        let tokio_info = deps.get_dependency("tokio").unwrap();
        assert!(tokio_info.declared);
        assert_eq!(tokio_info.version, Some("1.0".to_string()));
        assert_eq!(tokio_info.features, vec!["full".to_string()]);
    }

    #[test]
    fn test_dev_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"

[dev-dependencies]
tempfile = "3.0"
"#;

        fs::write(&cargo_toml, content).unwrap();

        let parser = CargoDependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_cargo_toml(&cargo_toml).unwrap();

        // Regular dependency should be declared
        assert!(deps.has_dependency("serde"));
        assert!(deps.get_dependency("serde").unwrap().declared);

        // Dev dependency should also be tracked (for validation purposes)
        assert!(deps.has_dependency("tempfile"));
        // Note: dev dependencies are marked as declared for validation purposes
        assert!(deps.get_dependency("tempfile").unwrap().declared);
    }

    #[test]
    fn test_workspace_dependencies() {
        let temp_dir = TempDir::new().unwrap();

        // Create workspace Cargo.toml
        let workspace_cargo = temp_dir.path().join("Cargo.toml");
        fs::write(
            &workspace_cargo,
            r#"
[workspace]
members = ["crate1", "crate2"]
"#,
        )
        .unwrap();

        // Create crate1
        let crate1_dir = temp_dir.path().join("crate1");
        fs::create_dir(&crate1_dir).unwrap();
        let crate1_cargo = crate1_dir.join("Cargo.toml");
        fs::write(
            &crate1_cargo,
            r#"
[package]
name = "crate1"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        // Create crate2
        let crate2_dir = temp_dir.path().join("crate2");
        fs::create_dir(&crate2_dir).unwrap();
        let crate2_cargo = crate2_dir.join("Cargo.toml");
        fs::write(
            &crate2_cargo,
            r#"
[package]
name = "crate2"
version = "0.1.0"

[dependencies]
tokio = "1.0"
"#,
        )
        .unwrap();

        let parser = CargoDependencyParser::new(temp_dir.path().to_path_buf());
        let workspace_deps = parser.parse_workspace_deps().unwrap();

        // Check crate1 dependencies
        let crate1_deps = workspace_deps
            .find_crate_deps(&crate1_dir.join("src/main.rs"))
            .unwrap();
        assert!(crate1_deps.has_dependency("serde"));
        assert!(!crate1_deps.has_dependency("tokio"));

        // Check crate2 dependencies
        let crate2_deps = workspace_deps
            .find_crate_deps(&crate2_dir.join("src/main.rs"))
            .unwrap();
        assert!(crate2_deps.has_dependency("tokio"));
        assert!(!crate2_deps.has_dependency("serde"));
    }
}
