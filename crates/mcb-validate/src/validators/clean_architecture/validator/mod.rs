//! Clean Architecture validator implementation

use std::path::PathBuf;

use super::violation::CleanArchitectureViolation;
use crate::config::CleanArchitectureRulesConfig;
use crate::traits::violation::Violation;
use crate::{Result, ValidationConfig};

/// Clean Architecture validator
pub struct CleanArchitectureValidator {
    _workspace_root: PathBuf,
    _rules: CleanArchitectureRulesConfig,
    _naming: crate::config::NamingRulesConfig,
}

impl CleanArchitectureValidator {
    /// Create a new architecture validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(
            &ValidationConfig::new(root),
            &file_config.rules.clean_architecture,
            &file_config.rules.naming,
        )
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(
        config: &ValidationConfig,
        rules: &CleanArchitectureRulesConfig,
        naming: &crate::config::NamingRulesConfig,
    ) -> Self {
        Self {
            _workspace_root: config.workspace_root.clone(),
            _rules: rules.clone(),
            _naming: naming.clone(),
        }
    }

    /// Run all architecture validations (returns typed violations)
    ///
    /// # Errors
    ///
    /// Returns an error if the validation process fails.
    pub fn validate_all(&self) -> Result<Vec<CleanArchitectureViolation>> {
        // Validation is fully driven by declarative rules (YAML) loaded by DeclarativeValidator.
        Ok(Vec::new())
    }
}

impl crate::traits::validator::Validator for CleanArchitectureValidator {
    fn name(&self) -> &'static str {
        "clean_architecture"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture compliance: layer boundaries, DI patterns, entity identity, value object immutability (Delegated to DeclarativeValidator)"
    }

    fn validate(&self, _config: &ValidationConfig) -> crate::Result<Vec<Box<dyn Violation>>> {
        Ok(Vec::new())
    }
}
