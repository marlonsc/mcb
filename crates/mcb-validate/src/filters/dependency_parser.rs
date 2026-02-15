//! Cargo.toml Dependency Parser
//!
//! Parses Cargo.toml files to extract declared dependencies for validation.
//! Used to check if libraries used in code are properly declared.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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
    #[must_use]
    pub fn has_dependency(&self, name: &str) -> bool {
        self.deps.contains_key(name)
    }

    /// Get dependency info
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Parse dependencies for the entire workspace
    ///
    /// # Errors
    ///
    /// Returns an error if Cargo.toml files cannot be read or parsed.
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

        Self::collect_cargo_dirs(
            &self.workspace_root,
            &self.workspace_root,
            0,
            3,
            &mut crates,
        );

        crates
    }

    fn collect_cargo_dirs(
        root: &Path,
        current: &Path,
        depth: usize,
        max_depth: usize,
        crates: &mut Vec<PathBuf>,
    ) {
        if depth > max_depth {
            return;
        }

        let Ok(entries) = fs::read_dir(current) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml")
                    && let Some(parent) = path.parent()
                    && parent != root
                {
                    crates.push(parent.to_path_buf());
                }
                continue;
            }

            if path.is_dir() {
                Self::collect_cargo_dirs(root, &path, depth + 1, max_depth, crates);
            }
        }
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
