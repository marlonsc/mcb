//! Service Port Interfaces
//!
//! Defines port interfaces for application services.
//! These traits are the contracts that services must implement,
//! following Clean Architecture principles where ports are defined
//! in the domain layer.

use async_trait::async_trait;
use std::path::Path;

use crate::error::Result;

// ============================================================================
// Validation Service Interface
// ============================================================================

/// Report containing validation results
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationReport {
    /// Total number of violations found
    pub total_violations: usize,
    /// Number of error-level violations
    pub errors: usize,
    /// Number of warning-level violations
    pub warnings: usize,
    /// Number of info-level violations
    pub infos: usize,
    /// All violations found
    pub violations: Vec<ViolationEntry>,
    /// Whether validation passed (no error-level violations)
    pub passed: bool,
}

/// A single violation entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct ViolationEntry {
    /// Unique violation ID (e.g., "CA001", "SOLID002")
    pub id: String,
    /// Category (e.g., "clean_architecture", "solid", "quality")
    pub category: String,
    /// Severity level: "ERROR", "WARNING", or "INFO"
    pub severity: String,
    /// File path where violation was found (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Human-readable description of the violation
    pub message: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

/// Architecture Validation Service Interface
///
/// Defines the contract for running architecture validation on a codebase.
/// Implementations should delegate to mcb-validate for actual validation logic.
#[async_trait]
pub trait ValidationServiceInterface: Send + Sync {
    /// Validate a workspace against architecture rules
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    /// * `validators` - Optional list of specific validators to run
    /// * `severity_filter` - Optional minimum severity filter ("error", "warning", or "info")
    ///
    /// # Returns
    ///
    /// A `ValidationReport` containing all violations found.
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport>;

    /// List available validator names
    ///
    /// Returns a list of all validator names that can be passed to `validate()`.
    async fn list_validators(&self) -> Result<Vec<String>>;
}

// ============================================================================
// Null Implementation (for testing and fallback)
// ============================================================================

/// Null validation service for testing or when validation feature is disabled
///
/// Returns an empty report with all checks passed. Used as fallback
/// when mcb-validate is not available.
pub struct NullValidationService;

impl NullValidationService {
    /// Create a new null validation service
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for NullValidationService {
    async fn validate(
        &self,
        _workspace_root: &Path,
        _validators: Option<&[String]>,
        _severity_filter: Option<&str>,
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

    async fn list_validators(&self) -> Result<Vec<String>> {
        Ok(vec![
            "clean_architecture".into(),
            "solid".into(),
            "quality".into(),
        ])
    }
}
