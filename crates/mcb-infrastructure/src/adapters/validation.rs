//! Validation Provider Adapter
//!
//! Implementation of mapping between domain and real validation providers.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::validation::{
    RuleInfo, ValidationOptions, ValidationProvider, ValidationReport, ValidatorInfo,
};
use std::path::Path;

/// Null validation provider for testing or fallback
pub struct NullValidationProvider;

impl NullValidationProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullValidationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationProvider for NullValidationProvider {
    fn provider_name(&self) -> &str {
        "null"
    }

    fn description(&self) -> &str {
        "Null validation provider (no-op)"
    }

    fn list_validators(&self) -> Vec<ValidatorInfo> {
        Vec::new()
    }

    fn get_rules(&self, _category: Option<&str>) -> Vec<RuleInfo> {
        Vec::new()
    }

    async fn validate(
        &self,
        _workspace_root: &Path,
        _options: ValidationOptions,
    ) -> Result<ValidationReport> {
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: Vec::new(),
            passed: true,
        })
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _options: ValidationOptions,
    ) -> Result<ValidationReport> {
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: Vec::new(),
            passed: true,
        })
    }

    fn can_validate(&self, _path: &Path) -> bool {
        false
    }

    fn supported_extensions(&self) -> &[&str] {
        &[]
    }
}
