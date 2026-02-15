use std::path::PathBuf;

use super::super::violation::ImplementationViolation;
use super::utils::{required_patterns, source_lines};
use crate::Result;
use crate::traits::violation::Severity;

pub fn validate_empty_catch_alls(
    files: &[(PathBuf, String)],
) -> Result<Vec<ImplementationViolation>> {
    let catchall_ids = [
        "IMPL001.catchall_empty",
        "IMPL001.catchall_unit",
        "IMPL001.catchall_ok_unit",
        "IMPL001.catchall_none",
        "IMPL001.catchall_continue",
    ];

    let compiled = required_patterns(catchall_ids.iter().copied())?;
    let mut violations = Vec::new();

    for (file_path, content) in files {
        for (line_num, trimmed) in source_lines(content) {
            for pattern in &compiled {
                if pattern.is_match(trimmed) {
                    violations.push(ImplementationViolation::EmptyCatchAll {
                        file: file_path.to_path_buf(),
                        line: line_num,
                        context: trimmed.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
    Ok(violations)
}
