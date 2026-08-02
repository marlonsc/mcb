//! Validation report formatting.

use std::path::Path;
use std::time::Duration;

use mcb_domain::ports::ValidationReport;

pub(super) fn build_validation_message(
    report: &ValidationReport,
    path: &Path,
    duration: Duration,
) -> String {
    let json_output = serde_json::json!({
        "workspace": path.display().to_string(),
        "passed": report.passed,
        "total_violations": report.total_violations,
        "errors": report.errors,
        "warnings": report.warnings,
        "infos": report.infos,
        "duration_secs": duration.as_secs_f64(),
        "violations": report.violations.iter().map(|v| {
            serde_json::json!({
                "id": v.id,
                "category": v.category,
                "severity": v.severity,
                "file": v.file,
                "line": v.line,
                "message": v.message,
                "suggestion": v.suggestion
            })
        }).collect::<Vec<_>>()
    });

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| {
        format!(
            "{{\"error\": \"Failed to serialize validation report\", \"path\": \"{}\"}}",
            path.display()
        )
    })
}

pub(super) fn build_validation_error_message(error: &str, path: &Path) -> String {
    let json_output = serde_json::json!({
        "error": error,
        "path": path.display().to_string(),
        "passed": false
    });

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| {
        format!(
            "{{\"error\": \"{}\", \"path\": \"{}\"}}",
            error,
            path.display()
        )
    })
}
