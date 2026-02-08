//! Mock Validation Service implementation

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::services::{ValidationReport, ValidationServiceInterface, ViolationEntry};

/// Mock implementation of ValidationServiceInterface for testing
pub struct MockValidationService {
    /// Pre-configured validation report
    pub report: Arc<Mutex<ValidationReport>>,
    /// List of available validators
    pub validators: Arc<Mutex<Vec<String>>>,
    /// Whether the next call should fail
    pub should_fail: Arc<AtomicBool>,
    /// Error message to return on failure
    pub error_message: Arc<Mutex<String>>,
}

impl MockValidationService {
    /// Create a new mock validation service
    pub fn new() -> Self {
        Self {
            report: Arc::new(Mutex::new(ValidationReport {
                total_violations: 0,
                errors: 0,
                warnings: 0,
                infos: 0,
                violations: Vec::new(),
                passed: true,
            })),
            validators: Arc::new(Mutex::new(vec![
                "clean_architecture".into(),
                "solid".into(),
                "quality".into(),
            ])),
            should_fail: Arc::new(AtomicBool::new(false)),
            error_message: Arc::new(Mutex::new("Simulated validation failure".to_string())),
        }
    }

    /// Configure the mock to return specific validation report
    pub fn with_report(self, report: ValidationReport) -> Self {
        *self.report.lock().expect("Lock poisoned") = report;
        self
    }

    /// Configure the mock to return specific validators
    pub fn with_validators(self, validators: Vec<String>) -> Self {
        *self.validators.lock().expect("Lock poisoned") = validators;
        self
    }

    /// Configure the mock to fail on next call
    pub fn with_failure(self, message: &str) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        *self.error_message.lock().expect("Lock poisoned") = message.to_string();
        self
    }

    /// Create a mock with violations for testing
    pub fn with_violations(violations: Vec<ViolationEntry>) -> Self {
        let errors = violations.iter().filter(|v| v.severity == "ERROR").count();
        let warnings = violations
            .iter()
            .filter(|v| v.severity == "WARNING")
            .count();
        let infos = violations.iter().filter(|v| v.severity == "INFO").count();

        Self::new().with_report(ValidationReport {
            total_violations: violations.len(),
            errors,
            warnings,
            infos,
            violations,
            passed: errors == 0,
        })
    }
}

impl Default for MockValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for MockValidationService {
    async fn validate(
        &self,
        _workspace_root: &Path,
        _validators: Option<&[String]>,
        _severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(self.report.lock().expect("Lock poisoned").clone())
    }

    async fn list_validators(&self) -> Result<Vec<String>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }

        Ok(self.validators.lock().expect("Lock poisoned").clone())
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(self.report.lock().expect("Lock poisoned").clone())
    }

    async fn get_rules(
        &self,
        _category: Option<&str>,
    ) -> Result<Vec<mcb_domain::ports::services::RuleInfo>> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(Vec::new())
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        _include_functions: bool,
    ) -> Result<mcb_domain::ports::services::ComplexityReport> {
        if self.should_fail.load(Ordering::SeqCst) {
            let msg = self.error_message.lock().expect("Lock poisoned").clone();
            return Err(mcb_domain::error::Error::internal(msg));
        }
        Ok(mcb_domain::ports::services::ComplexityReport {
            file: file_path.to_string_lossy().to_string(),
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 100.0,
            sloc: 0,
            functions: Vec::new(),
        })
    }
}
