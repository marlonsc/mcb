//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::violation::PatternViolation;
use mcb_domain::ports::validation::Severity;
use mcb_utils::constants::validate::COMMENT_PREFIX;
use mcb_utils::utils::regex::compile_regex;

/// Detects `std::result::Result<T, E>` where the error `E` is the spelled-out
/// domain error (`Error` / `mcb_domain::Error` / `crate::error::Error`) or
/// `String` — exactly the cases that should instead use the domain `Result`
/// alias. Foreign error types (`sea_orm::DbErr`, `regex::Error`, axum tuples,
/// generic parameters, `FromStr`-required signatures, alias definitions) are
/// necessary and left untouched. The error arg must close the type (`...>`),
/// which also excludes string literals mentioning the type.
pub fn check_result_types(path: &Path, content: &str) -> crate::Result<Vec<PatternViolation>> {
    let mut violations = Vec::new();

    let domain_error_result = compile_regex(
        r"std::result::Result\s*<.*,\s*(?:mcb_domain::error::Error|mcb_domain::Error|crate::error::Error|Error|String)\s*>",
    )?;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments and type-alias definitions (the alias itself must spell
        // out the std form).
        if trimmed.starts_with(COMMENT_PREFIX) || trimmed.contains("= std::result::Result") {
            continue;
        }

        if domain_error_result.is_match(trimmed) {
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
