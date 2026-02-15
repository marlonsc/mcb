use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::violation::PatternViolation;
use crate::traits::violation::Severity;

static STD_RESULT_PATTERN: OnceLock<Regex> = OnceLock::new();
static EXPLICIT_RESULT_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Checks for result type usage violations in a single file.
pub fn check_result_types(path: &Path, content: &str) -> Vec<PatternViolation> {
    let mut violations = Vec::new();

    // Pattern to find std::result::Result usage
    let std_result_pattern = STD_RESULT_PATTERN
        .get_or_init(|| Regex::new("std::result::Result<").expect("Invalid std result pattern"));

    // Pattern to find Result<T, E> with explicit error type (not crate::Result)
    let explicit_result_pattern = EXPLICIT_RESULT_PATTERN.get_or_init(|| {
        Regex::new(r"Result<[^,]+,\s*([A-Za-z][A-Za-z0-9_:]+)>")
            .expect("Invalid explicit result pattern")
    });

    // Skip error-related files (they define/extend error types)
    let file_name = path.file_name().and_then(|n| n.to_str());
    let parent_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str());
    if file_name.is_some_and(|n| n == "error.rs" || n == "error_ext.rs" || n.starts_with("error"))
        || parent_name.is_some_and(|p| p == "error")
    {
        return violations;
    }

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments and use statements
        if trimmed.starts_with("//") || trimmed.starts_with("use ") {
            continue;
        }

        // Check for std::result::Result
        if std_result_pattern.is_match(line) {
            violations.push(PatternViolation::RawResultType {
                file: path.to_path_buf(),
                line: line_num + 1,
                context: trimmed.to_owned(),
                suggestion: "crate::Result<T>".to_owned(),
                severity: Severity::Warning,
            });
        }

        // Check for Result<T, SomeError> with explicit error type
        if let Some(cap) = explicit_result_pattern.captures(line) {
            let error_type = cap.get(1).map_or("", |m| m.as_str());

            // Allow certain standard error types
            let allowed_errors = [
                "Error",
                "crate::Error",
                "crate::error::Error",
                "ValidationError",
                "std::io::Error",
                "anyhow::Error",
            ];

            if !allowed_errors.contains(&error_type)
                && !error_type.starts_with("crate::")
                && !error_type.starts_with("self::")
            {
                // This is informational - sometimes explicit error types are needed
                // We won't flag this as a violation for now
            }
        }
    }
    violations
}
