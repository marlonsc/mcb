use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::{Result, ValidationConfig};

use super::bypass::validate_bypass_boundaries;
use super::cargo::validate_cargo_dependencies;
use super::cycles::detect_circular_dependencies;
use super::uses::validate_use_statements;
use super::violation::DependencyViolation;

/// Validates Clean Architecture dependency rules across crates.
///
/// Ensures that crates only depend on allowed layers according to Clean Architecture principles.
/// Validates both Cargo.toml dependencies and use statements in source code.
pub struct DependencyValidator {
    pub(crate) config: ValidationConfig,
    pub(crate) allowed_deps: HashMap<String, HashSet<String>>,
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
            let rule_id = format!("CA{i:03}");
            if let Some(config_val) = PATTERNS.get_config(&rule_id)
                && let Some(crate_name) = config_val.get("crate_name").and_then(|v| v.as_str())
            {
                let deps: HashSet<String> = PATTERNS
                    .get_config_list(&rule_id, "allowed_dependencies")
                    .into_iter()
                    .collect();
                allowed_deps.insert(crate_name.to_owned(), deps);
            }
        }

        Self {
            config,
            allowed_deps,
        }
    }

    /// Run all dependency validations
    ///
    /// # Errors
    ///
    /// Returns an error if any dependency check fails.
    pub fn validate_all(&self) -> Result<Vec<DependencyViolation>> {
        let mut violations = Vec::new();
        violations.extend(validate_cargo_dependencies(self)?);
        violations.extend(validate_use_statements(self)?);
        violations.extend(detect_circular_dependencies(self)?);
        violations.extend(validate_bypass_boundaries(self)?);
        Ok(violations)
    }

    /// Validate Cargo.toml dependencies match Clean Architecture rules
    ///
    /// # Errors
    ///
    /// Returns an error if Cargo.toml parsing fails.
    pub fn validate_cargo_dependencies(&self) -> Result<Vec<DependencyViolation>> {
        validate_cargo_dependencies(self)
    }

    /// Validate no forbidden use statements in source code
    ///
    /// # Errors
    ///
    /// Returns an error if source file scanning fails.
    pub fn validate_use_statements(&self) -> Result<Vec<DependencyViolation>> {
        validate_use_statements(self)
    }

    /// Detect circular dependencies using topological sort
    ///
    /// # Errors
    ///
    /// Returns an error if dependency graph construction fails.
    pub fn detect_circular_dependencies(&self) -> Result<Vec<DependencyViolation>> {
        detect_circular_dependencies(self)
    }

    /// Validate anti-bypass boundaries from config.
    ///
    /// # Errors
    ///
    /// Returns an error if boundary configuration parsing fails.
    pub fn validate_bypass_boundaries(&self) -> Result<Vec<DependencyViolation>> {
        validate_bypass_boundaries(self)
    }
}

crate::impl_validator!(
    DependencyValidator,
    "dependency",
    "Validates Clean Architecture layer dependencies"
);
