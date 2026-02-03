//! Validation Service Implementation
//!
//! Provides architecture validation capabilities.
//! This is a stub implementation - actual validation logic is handled
//! by the mcb-validate crate at the infrastructure level.
//!
//! Note: mcb-application cannot depend on mcb-validate (Clean Architecture
//! violation - mcb-validate is dev tooling). Real validation is wired via
//! DI at the infrastructure layer.

use async_trait::async_trait;
use mcb_domain::error::Result;
use std::path::Path;

use crate::ports::services::{
    ComplexityReport, RuleInfo, ValidationReport, ValidationServiceInterface,
};

/// Validation service implementation
///
/// This is a no-op stub implementation. The real validation logic
/// is injected at runtime via mcb-infrastructure's DI system.
pub struct ValidationService;

impl ValidationService {
    /// Create a new validation service
    pub fn new() -> Self {
        Self
    }
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for ValidationService {
    async fn validate(
        &self,
        _workspace_root: &Path,
        _validators: Option<&[String]>,
        _severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        // Stub implementation - returns empty report
        // Real validation is wired at infrastructure layer
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: vec![],
            passed: true,
        })
    }

    async fn list_validators(&self) -> Result<Vec<String>> {
        Ok(vec![
            "clean_architecture".into(),
            "solid".into(),
            "quality".into(),
            "organization".into(),
            "kiss".into(),
            "naming".into(),
            "documentation".into(),
            "performance".into(),
            "async_patterns".into(),
        ])
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        // Stub implementation - returns empty report
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: vec![],
            passed: true,
        })
    }

    async fn get_rules(&self, _category: Option<&str>) -> Result<Vec<RuleInfo>> {
        // Stub implementation - returns empty list
        Ok(Vec::new())
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        _include_functions: bool,
    ) -> Result<ComplexityReport> {
        // Stub implementation - returns default values
        Ok(ComplexityReport {
            file: file_path.to_string_lossy().to_string(),
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 100.0,
            sloc: 0,
            functions: Vec::new(),
        })
    }
}
