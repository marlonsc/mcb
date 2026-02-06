//! `ValidationReport` and `Reporter` implementation.

use crate::violation_trait::Violation;
use crate::{
    AsyncViolation, CleanArchitectureViolation, ConfigQualityViolation, DependencyViolation,
    DocumentationViolation, ErrorBoundaryViolation, ImplementationViolation, KissViolation,
    NamingViolation, OrganizationViolation, PatternViolation, PerformanceViolation, PmatViolation,
    QualityViolation, RefactoringViolation, Severity, SolidViolation, TestQualityViolation,
    TestViolation,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::PathBuf;

use super::summary::ValidationSummary;

/// Section of violations: (section title, violations as trait objects)
type ViolationSection<'a> = (&'static str, Vec<&'a dyn Violation>);

/// Validation report containing all violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Timestamp of the validation run
    pub timestamp: String,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Summary statistics
    pub summary: ValidationSummary,
    /// Dependency violations
    pub dependency_violations: Vec<DependencyViolation>,
    /// Quality violations
    pub quality_violations: Vec<QualityViolation>,
    /// Pattern violations
    pub pattern_violations: Vec<PatternViolation>,
    /// Test organization violations
    pub test_violations: Vec<TestViolation>,
    /// Documentation violations
    pub documentation_violations: Vec<DocumentationViolation>,
    /// Naming violations
    pub naming_violations: Vec<NamingViolation>,
    /// SOLID principle violations
    pub solid_violations: Vec<SolidViolation>,
    /// Organization violations (file placement, centralization)
    pub organization_violations: Vec<OrganizationViolation>,
    /// KISS principle violations (complexity)
    pub kiss_violations: Vec<KissViolation>,
    /// Refactoring completeness violations
    pub refactoring_violations: Vec<RefactoringViolation>,
    /// Implementation quality violations
    pub implementation_violations: Vec<ImplementationViolation>,
    /// Performance pattern violations
    pub performance_violations: Vec<PerformanceViolation>,
    /// Async pattern violations
    pub async_violations: Vec<AsyncViolation>,
    /// Error boundary violations
    pub error_boundary_violations: Vec<ErrorBoundaryViolation>,
    /// PMAT integration violations
    pub pmat_violations: Vec<PmatViolation>,
    /// Clean Architecture layer boundary violations (CA001-CA009)
    pub clean_architecture_violations: Vec<CleanArchitectureViolation>,
    /// Test quality violations (ignored tests, todo in fixtures)
    pub test_quality_violations: Vec<TestQualityViolation>,
    /// Configuration quality violations (hardcoded values)
    pub config_quality_violations: Vec<ConfigQualityViolation>,
}

impl ValidationReport {
    /// Returns violation sections (title, violations) for dynamic report rendering.
    #[allow(clippy::too_many_lines)]
    fn violation_sections(&self) -> Vec<ViolationSection<'_>> {
        vec![
            (
                "Dependency",
                self.dependency_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Quality",
                self.quality_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Patterns",
                self.pattern_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Test Organization",
                self.test_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Documentation",
                self.documentation_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Naming",
                self.naming_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "SOLID",
                self.solid_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Organization",
                self.organization_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "KISS",
                self.kiss_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Refactoring",
                self.refactoring_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Implementation Quality",
                self.implementation_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Performance",
                self.performance_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Async Patterns",
                self.async_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Error Boundary",
                self.error_boundary_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "PMAT",
                self.pmat_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Clean Architecture",
                self.clean_architecture_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Test Quality",
                self.test_quality_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
            (
                "Config Quality",
                self.config_quality_violations
                    .iter()
                    .map(|v| v as &dyn Violation)
                    .collect(),
            ),
        ]
    }

    /// Collects error-level violation messages for CI output.
    fn collect_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (_, violations) in self.violation_sections() {
            for v in violations {
                if v.severity() == Severity::Error {
                    errors.push(format!("::error ::{v}"));
                }
            }
        }
        errors
    }

    /// Counts violations by severity (Error or Warning).
    fn count_by_severity(&self, severity: Severity) -> usize {
        self.violation_sections()
            .into_iter()
            .map(|(_, v)| v.iter().filter(|x| x.severity() == severity).count())
            .sum()
    }
}

/// Report generator
pub struct Reporter;

impl Reporter {
    /// Generate JSON report
    pub fn to_json(report: &ValidationReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate human-readable report
    pub fn to_human_readable(report: &ValidationReport) -> String {
        let mut output = String::new();
        output.push_str("=== Architecture Validation Report ===\n\n");
        let _ = writeln!(output, "Timestamp: {}", report.timestamp);
        let _ = writeln!(output, "Workspace: {}", report.workspace_root.display());
        let _ = writeln!(output);
        output.push_str("--- Summary ---\n");
        let _ = writeln!(
            output,
            "Total Violations: {}",
            report.summary.total_violations
        );
        for (label, count) in report.summary.summary_rows() {
            let _ = writeln!(output, "  {label:<17} {count}");
        }
        let _ = writeln!(output);
        let status = if report.summary.passed {
            "PASSED"
        } else {
            "FAILED"
        };
        let _ = writeln!(output, "Status: {status}");
        let _ = writeln!(output);
        for (title, violations) in report.violation_sections() {
            if violations.is_empty() {
                continue;
            }
            let _ = writeln!(output, "--- {title} Violations ---");
            for v in violations {
                let _ = writeln!(output, "  [{:?}] {}", v.severity(), v);
            }
            let _ = writeln!(output);
        }
        output
    }

    /// Generate CI summary (GitHub Actions format)
    pub fn to_ci_summary(report: &ValidationReport) -> String {
        let mut output = String::new();
        output.push_str("## Architecture Validation\n\n");
        let status = if report.summary.passed {
            "**Status:** :white_check_mark: PASSED"
        } else {
            "**Status:** :x: FAILED"
        };
        let _ = writeln!(output, "{status}\n");
        output.push_str("| Category | Count |\n");
        output.push_str("|----------|-------|\n");
        for (label, count) in report.summary.summary_rows() {
            let _ = writeln!(output, "| {label} | {count} |");
        }
        let _ = writeln!(
            output,
            "| **Total** | **{}** |",
            report.summary.total_violations
        );
        let _ = writeln!(output);
        let errors = report.collect_errors();
        if !errors.is_empty() {
            output.push_str("\n### Errors\n\n");
            for e in errors {
                let _ = writeln!(output, "{e}");
            }
        }
        output
    }

    /// Count error-level violations
    pub fn count_errors(report: &ValidationReport) -> usize {
        report.count_by_severity(Severity::Error)
    }

    /// Count warning-level violations
    pub fn count_warnings(report: &ValidationReport) -> usize {
        report.count_by_severity(Severity::Warning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QualityViolation;

    fn create_empty_report() -> ValidationReport {
        ValidationReport {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            workspace_root: PathBuf::from("/test"),
            summary: ValidationSummary {
                total_violations: 0,
                dependency_count: 0,
                quality_count: 0,
                pattern_count: 0,
                test_count: 0,
                documentation_count: 0,
                naming_count: 0,
                solid_count: 0,
                organization_count: 0,
                kiss_count: 0,
                refactoring_count: 0,
                implementation_count: 0,
                performance_count: 0,
                async_count: 0,
                error_boundary_count: 0,
                pmat_count: 0,
                clean_architecture_count: 0,
                test_quality_count: 0,
                config_quality_count: 0,
                passed: true,
            },
            dependency_violations: vec![],
            quality_violations: vec![],
            pattern_violations: vec![],
            test_violations: vec![],
            documentation_violations: vec![],
            naming_violations: vec![],
            solid_violations: vec![],
            organization_violations: vec![],
            kiss_violations: vec![],
            refactoring_violations: vec![],
            implementation_violations: vec![],
            performance_violations: vec![],
            async_violations: vec![],
            error_boundary_violations: vec![],
            pmat_violations: vec![],
            clean_architecture_violations: vec![],
            test_quality_violations: vec![],
            config_quality_violations: vec![],
        }
    }

    #[test]
    fn test_json_output() {
        let report = create_empty_report();
        let json = Reporter::to_json(&report);
        assert!(json.contains("timestamp"));
        assert!(json.contains("summary"));
        assert!(json.contains("passed"));
    }

    #[test]
    fn test_human_readable_output() {
        let report = create_empty_report();
        let output = Reporter::to_human_readable(&report);
        assert!(output.contains("Architecture Validation Report"));
        assert!(output.contains("Summary"));
        assert!(output.contains("PASSED"));
    }

    #[test]
    fn test_ci_summary_output() {
        let report = create_empty_report();
        let output = Reporter::to_ci_summary(&report);
        assert!(output.contains("Architecture Validation"));
        assert!(output.contains(":white_check_mark:"));
        assert!(output.contains("| Category | Count |"));
    }

    #[test]
    fn test_error_counting() {
        use crate::constants::REPORT_TEST_PENDING_LABEL;
        let mut report = create_empty_report();
        report
            .quality_violations
            .push(QualityViolation::UnwrapInProduction {
                file: PathBuf::from("/test.rs"),
                line: 1,
                context: "test".to_string(),
                severity: Severity::Error,
            });
        report
            .quality_violations
            .push(QualityViolation::TodoComment {
                file: PathBuf::from("/test.rs"),
                line: 2,
                content: REPORT_TEST_PENDING_LABEL.to_string(),
                severity: Severity::Info,
            });
        assert_eq!(Reporter::count_errors(&report), 1);
        assert_eq!(Reporter::count_warnings(&report), 0);
    }
}
