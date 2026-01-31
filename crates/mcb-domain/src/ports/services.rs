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

/// Information about a validation rule
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleInfo {
    /// Rule ID (e.g., "CA001")
    pub id: String,
    /// Rule category
    pub category: String,
    /// Rule severity (error, warning, info)
    pub severity: String,
    /// Human-readable description
    pub description: String,
    /// Engine that executes this rule
    pub engine: String,
}

/// Code complexity metrics report
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplexityReport {
    /// File path
    pub file: String,
    /// Cyclomatic complexity
    pub cyclomatic: f64,
    /// Cognitive complexity
    pub cognitive: f64,
    /// Maintainability index (0-100)
    pub maintainability_index: f64,
    /// Source lines of code
    pub sloc: usize,
    /// Function-level metrics (if requested)
    pub functions: Vec<FunctionComplexity>,
}

/// Function-level complexity metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct FunctionComplexity {
    /// Function name
    pub name: String,
    /// Start line number
    pub line: usize,
    /// Cyclomatic complexity
    pub cyclomatic: f64,
    /// Cognitive complexity
    pub cognitive: f64,
    /// Source lines of code
    pub sloc: usize,
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

    /// Validate a single file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to validate
    /// * `validators` - Optional list of specific validators to run
    ///
    /// # Returns
    ///
    /// A `ValidationReport` containing violations found in the file.
    async fn validate_file(
        &self,
        file_path: &Path,
        validators: Option<&[String]>,
    ) -> Result<ValidationReport>;

    /// Get available validation rules
    ///
    /// # Arguments
    ///
    /// * `category` - Optional category filter (e.g., "clean_architecture")
    ///
    /// # Returns
    ///
    /// List of rule information.
    async fn get_rules(&self, category: Option<&str>) -> Result<Vec<RuleInfo>>;

    /// Analyze code complexity using RCA metrics
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to analyze
    /// * `include_functions` - Whether to include function-level metrics
    ///
    /// # Returns
    ///
    /// Complexity report with metrics.
    async fn analyze_complexity(
        &self,
        file_path: &Path,
        include_functions: bool,
    ) -> Result<ComplexityReport>;
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

    async fn validate_file(
        &self,
        _file_path: &Path,
        _validators: Option<&[String]>,
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

    async fn get_rules(&self, _category: Option<&str>) -> Result<Vec<RuleInfo>> {
        Ok(Vec::new())
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        _include_functions: bool,
    ) -> Result<ComplexityReport> {
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
