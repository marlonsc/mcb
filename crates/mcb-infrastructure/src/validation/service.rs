//! Validation Service Implementation
//!
//! Implements `ValidationServiceInterface` using mcb-validate for
//! architecture validation.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::services::{ValidationReport, ValidationServiceInterface, ViolationEntry};
use std::path::Path;

/// Infrastructure validation service using mcb-validate
///
/// This is the real implementation that delegates to mcb-validate.
/// Named differently from mcb-application's stub to avoid REF002 violation.
pub struct InfraValidationService;

impl InfraValidationService {
    /// Create a new validation service
    pub fn new() -> Self {
        Self
    }
}

impl Default for InfraValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for InfraValidationService {
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        run_validation(workspace_root, validators, severity_filter)
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
            "dependencies".into(),
            "patterns".into(),
            "tests".into(),
        ])
    }
}

fn run_validation(
    workspace_root: &Path,
    validators: Option<&[String]>,
    severity_filter: Option<&str>,
) -> Result<ValidationReport> {
    use mcb_validate::{ArchitectureValidator, ValidationConfig};

    let config = ValidationConfig::new(workspace_root);
    let mut validator = ArchitectureValidator::with_config(config);

    let report = if let Some(names) = validators {
        let names_ref: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        validator
            .validate_named(&names_ref)
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?
    } else {
        validator
            .validate_all()
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?
    };

    Ok(convert_report(report, severity_filter))
}

fn convert_report(
    report: mcb_validate::GenericReport,
    severity_filter: Option<&str>,
) -> ValidationReport {
    let min_severity = match severity_filter {
        Some("error") => 0,
        Some("warning") => 1,
        _ => 2,
    };

    let all_violations: Vec<mcb_validate::ViolationEntry> = report
        .violations_by_category
        .into_values()
        .flatten()
        .collect();

    let violations: Vec<ViolationEntry> = all_violations
        .into_iter()
        .filter(|v| {
            let severity_level = match v.severity.as_str() {
                "ERROR" => 0,
                "WARNING" => 1,
                _ => 2,
            };
            severity_level <= min_severity
        })
        .map(|v| ViolationEntry {
            id: v.id,
            category: v.category,
            severity: v.severity,
            file: v.file.map(|p| p.to_string_lossy().to_string()),
            line: v.line,
            message: v.message,
            suggestion: v.suggestion,
        })
        .collect();

    let errors = violations.iter().filter(|v| v.severity == "ERROR").count();

    ValidationReport {
        total_violations: violations.len(),
        errors,
        warnings: violations
            .iter()
            .filter(|v| v.severity == "WARNING")
            .count(),
        infos: violations.iter().filter(|v| v.severity == "INFO").count(),
        violations,
        passed: errors == 0,
    }
}
