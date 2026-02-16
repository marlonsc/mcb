//! Linter Parsers Module
//!
//! Functions for parsing linter output formats.

use std::path::Path;

use super::constants::CLIPPY_PREFIX;
use super::types::{ClippyOutput, LintViolation, RuffViolation};

/// Execute linter command
///
/// # Errors
///
/// Returns an error if the linter process fails to spawn or execute.
pub async fn run_linter_command(
    linter: crate::linters::types::LinterType,
    files: &[&Path],
) -> crate::Result<String> {
    let output = tokio::process::Command::new(linter.command())
        .args(linter.args(files))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // Even if command fails, we might have partial output
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Parse Ruff JSON output
///
/// Ruff outputs JSON in array format when using `--output-format=json`:
/// ```json
/// [
///   {
///     "code": "F401",
///     "message": "...",
///     "filename": "...",
///     "location": {"row": 1, "column": 1},
///     ...
///   }
/// ]
/// ```
#[must_use]
pub fn parse_ruff_output(output: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    if let Ok(ruff_violations) = serde_json::from_str::<Vec<RuffViolation>>(output) {
        for ruff_violation in ruff_violations {
            violations.push(LintViolation {
                rule: ruff_violation.code.clone(),
                file_path_cache: Some(std::path::PathBuf::from(&ruff_violation.filename)),
                file: ruff_violation.filename,
                line: ruff_violation.location.row,
                column: ruff_violation.location.column,
                message: ruff_violation.message,
                severity: map_ruff_severity(&ruff_violation.code),
                category: "quality".to_owned(),
            });
        }
    }

    violations
}

/// Parse Clippy JSON output
///
/// Clippy outputs JSON lines with different "reason" types:
/// - "compiler-message": contains lint/warning/error messages
/// - "compiler-artifact": build artifacts (ignore)
/// - "build-finished": build completion (ignore)
///
/// The message structure for compiler-message:
/// ```json
/// {
///   "reason": "compiler-message",
///   "message": {
///     "code": {"code": "clippy::unwrap_used", "explanation": null},
///     "level": "warning",
///     "message": "...",
///     "spans": [{"file_name": "...", "line_start": 1, "column_start": 1, ...}]
///   }
/// }
/// ```
#[must_use]
pub fn parse_clippy_output(output: &str) -> Vec<LintViolation> {
    let mut violations = Vec::new();

    // Parse JSON lines
    for line in output.lines() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Try to parse as ClippyOutput (with reason field)
        if let Ok(clippy_output) = serde_json::from_str::<ClippyOutput>(line) {
            // Only process compiler-message reason
            if clippy_output.reason != super::constants::CLIPPY_REASON_COMPILER_MESSAGE {
                continue;
            }

            let msg = &clippy_output.message;

            // Skip messages without primary spans
            let Some(span) = msg.spans.iter().find(|s| s.is_primary) else {
                continue;
            };

            // Extract the code (either from nested structure or direct)
            let raw_code = msg
                .code
                .as_ref()
                .map(|c| c.code.clone())
                .unwrap_or_default();

            // Skip if no rule code (likely a build error, not a lint)
            if raw_code.is_empty() {
                continue;
            }

            // Normalize rule code: ensure clippy:: prefix for consistency
            let rule_code = if raw_code.starts_with(CLIPPY_PREFIX) {
                raw_code
            } else {
                format!("{CLIPPY_PREFIX}{raw_code}")
            };

            violations.push(LintViolation {
                rule: rule_code.clone(),
                file_path_cache: Some(std::path::PathBuf::from(&span.file_name)),
                file: span.file_name.clone(),
                line: span.line_start,
                column: span.column_start,
                message: msg.message.clone(),
                severity: map_clippy_level(&msg.level),
                category: if rule_code.contains("clippy") {
                    "quality"
                } else {
                    "correctness"
                }
                .to_owned(),
            });
        }
    }

    violations
}

/// Find project root from files (simplified)
#[must_use]
pub fn find_project_root(files: &[&Path]) -> Option<std::path::PathBuf> {
    // Simple heuristic: go up until we find Cargo.toml
    if let Some(first_file) = files.first() {
        let mut current = first_file.parent()?;
        loop {
            if current.join(super::constants::CARGO_TOML_FILENAME).exists() {
                return Some(current.to_path_buf());
            }
            current = current.parent()?;
        }
    }
    None
}

/// Map Ruff severity
#[must_use]
pub fn map_ruff_severity(code: &str) -> String {
    match code.chars().next() {
        Some('F' | 'E') => "error".to_owned(), // Pyflakes / pycodestyle errors
        Some('W') => "warning".to_owned(),     // pycodestyle warnings
        Some(_) | None => "info".to_owned(),
    }
}

/// Map Clippy level
#[must_use]
pub fn map_clippy_level(level: &str) -> String {
    match level {
        "error" => "error".to_owned(),
        "warning" => "warning".to_owned(),
        _ => "info".to_owned(), // note, help, and others
    }
}
