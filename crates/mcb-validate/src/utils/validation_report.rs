use crate::GenericReport;
use mcb_domain::ports::{ValidationReport, ViolationEntry};

fn severity_threshold(severity_filter: Option<&str>) -> u8 {
    match severity_filter.map(str::to_ascii_lowercase).as_deref() {
        Some("error") => 0,
        Some("warning") => 1,
        _ => 2,
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "ERROR" => 0,
        "WARNING" => 1,
        _ => 2,
    }
}

/// Convert a list of violations into a comprehensive validation report.
#[must_use]
pub fn from_violations(violations: Vec<ViolationEntry>) -> ValidationReport {
    let (errors, warnings, infos) =
        violations
            .iter()
            .fold((0, 0, 0), |(e, w, i), v| match v.severity.as_str() {
                "ERROR" => (e + 1, w, i),
                "WARNING" => (e, w + 1, i),
                _ => (e, w, i + 1),
            });

    ValidationReport {
        total_violations: violations.len(),
        errors,
        warnings,
        infos,
        violations,
        passed: errors == 0,
    }
}

/// Convert a generic internal report to a public validation report, applying severity filters.
#[must_use]
pub fn from_generic_report(
    report: GenericReport,
    severity_filter: Option<&str>,
    include_suggestions: bool,
) -> ValidationReport {
    let min_severity = severity_threshold(severity_filter);

    let violations = report
        .violations_by_category
        .into_values()
        .flatten()
        .filter(|violation| severity_rank(&violation.severity) <= min_severity)
        .map(|mut violation| {
            if !include_suggestions {
                violation.suggestion = None;
            }
            violation
        })
        .collect();

    from_violations(violations)
}
