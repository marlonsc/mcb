//! Clean Architecture validator implementation

mod boundaries;
mod domain;

use std::path::PathBuf;

use super::violation::CleanArchitectureViolation;
use crate::config::CleanArchitectureRulesConfig;
use crate::traits::violation::Violation;
use crate::{Result, ValidationConfig};

/// Clean Architecture validator
pub struct CleanArchitectureValidator {
    workspace_root: PathBuf,
    rules: CleanArchitectureRulesConfig,
    naming: crate::config::NamingRulesConfig,
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
    pub fn with_config(
        config: &ValidationConfig,
        rules: &CleanArchitectureRulesConfig,
        naming: &crate::config::NamingRulesConfig,
    ) -> Self {
        Self {
            workspace_root: config.workspace_root.clone(),
            rules: rules.clone(),
            naming: naming.clone(),
        }
    }

    /// Run all architecture validations (returns typed violations)
    pub fn validate_all(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_server_layer_boundaries()?);
        violations.extend(self.validate_handler_injection()?);
        violations.extend(self.validate_entity_identity()?);
        violations.extend(self.validate_value_object_immutability()?);
        // ADR-029: Hexagonal architecture validations
        violations.extend(self.validate_ca007_infrastructure_concrete_imports()?);
        violations.extend(self.validate_ca008_application_port_imports()?);
        // CA009: Infrastructure should NOT import from Application layer
        violations.extend(self.validate_ca009_infrastructure_imports_application()?);
        Ok(violations)
    }

    /// Run all validations (returns boxed violations for Validator trait)
    fn validate_boxed(&self) -> Result<Vec<Box<dyn Violation>>> {
        let typed_violations = self.validate_all()?;
        Ok(typed_violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}

impl crate::traits::validator::Validator for CleanArchitectureValidator {
    fn name(&self) -> &'static str {
        "clean_architecture"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture compliance: layer boundaries, DI patterns, entity identity, value object immutability"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        self.validate_boxed().map_err(|e| anyhow::anyhow!("{e}"))
    }
}
