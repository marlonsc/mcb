//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::violation::PatternViolation;
use mcb_domain::ports::validation::Severity;
use mcb_utils::constants::validate::COMMENT_PREFIX;

/// Detects `std::result::Result` usage that should use `crate::Result`.
pub fn check_result_types(path: &Path, content: &str) -> crate::Result<Vec<PatternViolation>> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        if trimmed.contains("std::result::Result") {
            let context = trimmed.chars().take(80).collect::<String>();
            violations.push(PatternViolation::RawResultType {
                file: path.to_path_buf(),
                line: line_num + 1,
                context,
                suggestion: "crate::Result or domain Result alias".to_owned(),
                severity: Severity::Warning,
            });
        }
    }

    Ok(violations)
}
