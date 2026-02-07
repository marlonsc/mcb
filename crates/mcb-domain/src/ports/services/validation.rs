use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

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
#[async_trait]
pub trait ValidationServiceInterface: Send + Sync {
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport>;

    async fn list_validators(&self) -> Result<Vec<String>>;

    async fn validate_file(
        &self,
        file_path: &Path,
        validators: Option<&[String]>,
    ) -> Result<ValidationReport>;

    async fn get_rules(&self, category: Option<&str>) -> Result<Vec<RuleInfo>>;

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        include_functions: bool,
    ) -> Result<ComplexityReport>;
}
