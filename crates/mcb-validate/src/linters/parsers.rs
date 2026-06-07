//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Linter Parsers Module
//!
//! Functions for parsing linter output formats.

use std::path::Path;

use super::types::{ClippyOutput, LintViolation, RuffViolation};
use crate::constants::linters::CLIPPY_PREFIX;

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

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
    serde_json::from_str::<Vec<RuffViolation>>(output)
        // INTENTIONAL: JSON parse fallback for linter output; empty array is safe default
        .unwrap_or_default()
        .into_iter()
        .map(|ruff_violation| LintViolation {
            rule: ruff_violation.code.clone(),
            file_path_cache: Some(std::path::PathBuf::from(&ruff_violation.filename)),
            file: ruff_violation.filename,
            line: ruff_violation.location.row,
            column: ruff_violation.location.column,
            message: ruff_violation.message,
            severity: map_ruff_severity(&ruff_violation.code),
            category: "quality".to_owned(),
        })
        .collect()
}

fn normalize_clippy_rule(raw_code: String) -> Option<String> {
    if raw_code.is_empty() {
        return None;
    }

    Some(if raw_code.starts_with(CLIPPY_PREFIX) {
        raw_code
    } else {
        format!("{CLIPPY_PREFIX}{raw_code}")
    })
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
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<ClippyOutput>(line).ok())
        .filter(|clippy_output| {
            clippy_output.reason == crate::constants::linters::CLIPPY_REASON_COMPILER_MESSAGE
        })
        .filter_map(|clippy_output| {
            let msg = clippy_output.message;
            let span = msg.spans.into_iter().find(|s| s.is_primary)?;
            // INTENTIONAL: Clippy JSON parse fallback; empty array is safe default
            let raw_code = msg.code.map(|c| c.code).unwrap_or_default();
            let rule_code = normalize_clippy_rule(raw_code)?;
            let category = if rule_code.contains("clippy") {
                "quality"
            } else {
                "correctness"
            }
            .to_owned();

            Some(LintViolation {
                rule: rule_code,
                file_path_cache: Some(std::path::PathBuf::from(&span.file_name)),
                file: span.file_name,
                line: span.line_start,
                column: span.column_start,
                message: msg.message,
                severity: map_clippy_level(&msg.level),
                category,
            })
        })
        .collect()
}

/// Find project root from files (simplified)
#[must_use]
pub fn find_project_root(files: &[&Path]) -> Option<std::path::PathBuf> {
    // Simple heuristic: go up until we find Cargo.toml
    if let Some(first_file) = files.first() {
        let mut current = first_file.parent()?;
        loop {
            if current
                .join(crate::constants::linters::CARGO_TOML_FILENAME)
                .exists()
            {
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
