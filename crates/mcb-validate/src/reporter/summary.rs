//! Validation summary types for report generation.

use serde::{Deserialize, Serialize};

/// Summary row for table rendering: (display label, count)
pub(crate) type SummaryRow = (&'static str, usize);

/// Summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of violations
    pub total_violations: usize,
    /// Number of dependency violations
    pub dependency_count: usize,
    /// Number of quality violations
    pub quality_count: usize,
    /// Number of pattern violations
    pub pattern_count: usize,
    /// Number of test organization violations
    pub test_count: usize,
    /// Number of documentation violations
    pub documentation_count: usize,
    /// Number of naming violations
    pub naming_count: usize,
    /// Number of SOLID principle violations
    pub solid_count: usize,
    /// Number of organization violations
    pub organization_count: usize,
    /// Number of KISS principle violations
    pub kiss_count: usize,
    /// Number of refactoring completeness violations
    pub refactoring_count: usize,
    /// Number of implementation quality violations
    pub implementation_count: usize,
    /// Number of performance pattern violations
    pub performance_count: usize,
    /// Number of async pattern violations
    pub async_count: usize,
    /// Number of error boundary violations
    pub error_boundary_count: usize,
    /// Number of PMAT integration violations
    pub pmat_count: usize,
    /// Number of Clean Architecture violations (CA001-CA009)
    pub clean_architecture_count: usize,
    /// Number of test quality violations
    pub test_quality_count: usize,
    /// Number of configuration quality violations
    pub config_quality_count: usize,
    /// Whether validation passed (no error-level violations)
    pub passed: bool,
}

impl ValidationSummary {
    /// Returns summary rows for table rendering (label, count) in display order.
    #[must_use]
    pub fn summary_rows(&self) -> Vec<SummaryRow> {
        vec![
            ("Dependency", self.dependency_count),
            ("Quality", self.quality_count),
            ("Patterns", self.pattern_count),
            ("Tests", self.test_count),
            ("Documentation", self.documentation_count),
            ("Naming", self.naming_count),
            ("SOLID", self.solid_count),
            ("Organization", self.organization_count),
            ("KISS", self.kiss_count),
            ("Refactoring", self.refactoring_count),
            ("Implementation", self.implementation_count),
            ("Performance", self.performance_count),
            ("Async", self.async_count),
            ("ErrorBoundary", self.error_boundary_count),
            ("PMAT", self.pmat_count),
            ("CleanArch", self.clean_architecture_count),
            ("TestQuality", self.test_quality_count),
            ("ConfigQuality", self.config_quality_count),
        ]
    }
}
