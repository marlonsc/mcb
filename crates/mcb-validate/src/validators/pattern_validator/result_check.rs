//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::violation::PatternViolation;
use mcb_domain::ports::validation::Severity;
use mcb_utils::constants::validate::COMMENT_PREFIX;
use mcb_utils::utils::regex::compile_regex;
use regex::Regex;

/// True when `std::result::Result` on this line is necessary rather than a
/// spelled-out domain result: alias definitions, `FromStr` signatures, method
/// path references (`Result::ok`), turbofish collects, and generic-parameter
/// error types (matched by `generic_error`) all require the fully-qualified std
/// form.
fn is_necessary_std_result(trimmed: &str, generic_error: &Regex) -> bool {
    (trimmed.contains("type ") && trimmed.contains("= std::result::Result"))
        || trimmed.contains("fn from_str")
        || trimmed.contains("std::result::Result::")
        || trimmed.contains(".collect::<")
        || generic_error.is_match(trimmed)
}

/// Detects `std::result::Result` usage that should use `crate::Result`.
pub fn check_result_types(path: &Path, content: &str) -> crate::Result<Vec<PatternViolation>> {
    let mut violations = Vec::new();

    // Error arg is a bare generic type parameter (short all-uppercase id) that
    // cannot use a domain alias, e.g. `std::result::Result<T, E>`.
    let generic_error = compile_regex(r"std::result::Result\s*<[^<>]*,\s*[A-Z][A-Z0-9]?\s*>")?;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        if trimmed.contains("std::result::Result")
            && !is_necessary_std_result(trimmed, &generic_error)
        {
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
