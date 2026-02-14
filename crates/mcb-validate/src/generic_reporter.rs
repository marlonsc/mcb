//! Generic Reporter
//!
//! This module provides the `GenericReporter` which facilitates the generation of
//! validation reports in multiple formats. It centralizes the reporting logic
//! for all types of architectural and code quality violations.

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::Severity;
use crate::traits::violation::{Violation, ViolationCategory};

/// Report containing all violations with summary
#[derive(Debug, Clone, Serialize)]
pub struct GenericReport {
    /// Timestamp of the validation run
    pub timestamp: String,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Summary statistics
    pub summary: GenericSummary,
    /// All violations grouped by category
    pub violations_by_category: HashMap<String, Vec<ViolationEntry>>,
}

/// Summary of validation results
#[derive(Debug, Clone, Serialize)]
pub struct GenericSummary {
    /// Total number of violations
    pub total_violations: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of info messages
    pub infos: usize,
    /// Violations per category
    pub by_category: HashMap<String, usize>,
    /// Whether validation passed (no error-level violations)
    pub passed: bool,
}

/// Serializable violation entry.
///
/// Serializable violation entry.
#[derive(Debug, Clone, Serialize)]
pub struct ViolationEntry {
    /// Unique violation ID
    pub id: String,
    /// Category
    pub category: String,
    /// Severity
    pub severity: String,
    /// File path (if applicable)
    pub file: Option<PathBuf>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Human-readable message
    pub message: String,
    /// Suggested fix (if applicable)
    pub suggestion: Option<String>,
}

impl ViolationEntry {
    /// Create from a Violation trait object
    pub fn from_violation(v: &dyn Violation) -> Self {
        Self {
            id: v.id().to_string(),
            category: v.category().to_string(),
            severity: v.severity().to_string(),
            file: v.file().cloned(),
            line: v.line(),
            message: v.message(),
            suggestion: v.suggestion(),
        }
    }
}

/// Generic reporter for violations
pub struct GenericReporter;

impl GenericReporter {
    /// Create a report from violations
    pub fn create_report(
        violations: &[Box<dyn Violation>],
        workspace_root: PathBuf,
    ) -> GenericReport {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        // Count by severity
        let (errors, warnings, infos) =
            violations
                .iter()
                .fold((0, 0, 0), |(e, w, i), v| match v.severity() {
                    Severity::Error => (e + 1, w, i),
                    Severity::Warning => (e, w + 1, i),
                    Severity::Info => (e, w, i + 1),
                });

        // Group by category
        let mut by_category: HashMap<String, Vec<ViolationEntry>> = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        for v in violations {
            let category_name = v.category().to_string();
            let entry = ViolationEntry::from_violation(v.as_ref());

            by_category
                .entry(category_name.clone())
                .or_default()
                .push(entry);

            *category_counts.entry(category_name).or_default() += 1;
        }

        GenericReport {
            timestamp,
            workspace_root,
            summary: GenericSummary {
                total_violations: violations.len(),
                errors,
                warnings,
                infos,
                by_category: category_counts,
                passed: errors == 0,
            },
            violations_by_category: by_category,
        }
    }

    /// Generate JSON report
    pub fn to_json(violations: &[Box<dyn Violation>], workspace_root: PathBuf) -> String {
        let report = Self::create_report(violations, workspace_root);
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate human-readable report.
    ///
    /// Generate human-readable report.
    pub fn to_human_readable(violations: &[Box<dyn Violation>], workspace_root: PathBuf) -> String {
        let report = Self::create_report(violations, workspace_root);
        let mut output = String::new();

        output.push_str("=== Architecture Validation Report ===\n\n");
        let _ = writeln!(output, "Timestamp: {}", report.timestamp);
        let _ = writeln!(output, "Workspace: {}", report.workspace_root.display());
        let _ = writeln!(output);

        // Summary
        output.push_str("--- Summary ---\n");
        let _ = writeln!(
            output,
            "Total Violations: {} ({} errors, {} warnings, {} info)",
            report.summary.total_violations,
            report.summary.errors,
            report.summary.warnings,
            report.summary.infos
        );
        let _ = writeln!(
            output,
            "Status: {}",
            if report.summary.passed {
                "PASSED"
            } else {
                "FAILED"
            }
        );
        let _ = writeln!(output);

        // By category
        if !report.violations_by_category.is_empty() {
            output.push_str("--- Violations by Category ---\n\n");

            // Sort categories for consistent output
            let mut categories: Vec<_> = report.violations_by_category.keys().collect();
            categories.sort();

            for category in categories {
                let violations = &report.violations_by_category[category];
                if violations.is_empty() {
                    continue;
                }

                let _ = writeln!(output, "=== {} ({}) ===", category, violations.len());

                for v in violations {
                    let location = match (&v.file, v.line) {
                        (Some(f), Some(l)) => format!("{}:{}", f.display(), l),
                        (Some(f), None) => f.display().to_string(),
                        _ => "unknown".to_string(),
                    };

                    let _ = writeln!(
                        output,
                        "  [{:>7}] [{}] {} - {}",
                        v.severity, v.id, location, v.message
                    );

                    if let Some(ref suggestion) = v.suggestion {
                        let _ = writeln!(output, "            -> {suggestion}");
                    }
                }
                output.push('\n');
            }
        }

        output
    }

    /// Generate CI summary (GitHub Actions format).
    pub fn to_ci_summary(violations: &[Box<dyn Violation>]) -> String {
        let mut output = String::new();

        for v in violations {
            let level = match v.severity() {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => continue, // Info messages are not reported in CI
            };

            if let (Some(file), Some(line)) = (v.file(), v.line()) {
                let _ = writeln!(
                    output,
                    "::{} file={},line={}::[{}] {}",
                    level,
                    file.display(),
                    line,
                    v.id(),
                    v.message()
                );
            } else if let Some(file) = v.file() {
                let _ = writeln!(
                    output,
                    "::{} file={}::[{}] {}",
                    level,
                    file.display(),
                    v.id(),
                    v.message()
                );
            } else {
                let _ = writeln!(output, "::{} ::[{}] {}", level, v.id(), v.message());
            }
        }

        output
    }

    /// Count violations by severity
    pub fn count_by_severity(violations: &[Box<dyn Violation>]) -> (usize, usize, usize) {
        violations
            .iter()
            .fold((0, 0, 0), |(e, w, i), v| match v.severity() {
                Severity::Error => (e + 1, w, i),
                Severity::Warning => (e, w + 1, i),
                Severity::Info => (e, w, i + 1),
            })
    }

    /// Filter violations by category
    pub fn filter_by_category(
        violations: Vec<Box<dyn Violation>>,
        category: ViolationCategory,
    ) -> Vec<Box<dyn Violation>> {
        violations
            .into_iter()
            .filter(|v| v.category() == category)
            .collect()
    }

    /// Filter violations by severity
    pub fn filter_by_severity(
        violations: Vec<Box<dyn Violation>>,
        severity: Severity,
    ) -> Vec<Box<dyn Violation>> {
        violations
            .into_iter()
            .filter(|v| v.severity() == severity)
            .collect()
    }
}
