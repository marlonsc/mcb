//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
use std::path::Path;

use regex::Regex;

use super::violation::OrganizationViolation;
use crate::filters::LanguageId;
use crate::scan::{for_each_crate_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use mcb_utils::constants::validate::{ALLOWED_MAGIC_NUMBERS, MAGIC_NUMBER_REGEX};
use mcb_utils::constants::validate::{
    ATTRIBUTE_PREFIX, CONST_DECLARATION_PREFIXES, CONSTANTS_FILE_KEYWORDS, DOC_COMMENT_PREFIX,
    MODULE_DOC_PREFIX,
};
use mcb_utils::utils::regex::compile_regex;

/// Scans for numeric literals that should be extracted as named constants.
///
/// # Errors
///
/// Returns an error if file scanning or reading fails.
pub fn validate_magic_numbers(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    // Pattern for numeric literals: 5+ digits (skip 4-digit numbers to reduce noise)
    let magic_pattern = compile_regex(MAGIC_NUMBER_REGEX)?;

    for_each_crate_file(
        config,
        Some(LanguageId::Rust),
        |entry, _src_dir, _crate_name| {
            let path = &entry.absolute_path;
            // Skip constants.rs files (they're allowed to have numbers)
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name.is_some_and(|n| CONSTANTS_FILE_KEYWORDS.iter().any(|k| n.contains(k))) {
                return Ok(());
            }

            // Skip test files
            if path.to_str().is_none_or(is_test_path) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            collect_magic_numbers(path, &content, &magic_pattern, &mut violations);
            Ok(())
        },
    )?;

    Ok(violations)
}

/// Returns `true` for lines exempt from magic-number checks (const
/// declarations, attributes, doc comments, and assertions).
fn skip_magic_line(trimmed: &str) -> bool {
    CONST_DECLARATION_PREFIXES
        .iter()
        .any(|p| trimmed.starts_with(p))
        || [ATTRIBUTE_PREFIX, DOC_COMMENT_PREFIX, MODULE_DOC_PREFIX]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
        || trimmed.contains("assert")
}

/// Returns `true` when `num` is allowed in `line` (whitelisted, or part of a
/// digit-separated literal).
fn is_allowed_magic(num: &str, line: &str) -> bool {
    let segmented = format!("{}_{}", &num[..num.len().min(3)], &num[num.len().min(3)..]);
    ALLOWED_MAGIC_NUMBERS.contains(&num)
        || line.contains(&format!("_{num}"))
        || line.contains(&format!("{num}_"))
        || line.contains(&segmented)
}

/// Push a `MagicNumber` violation for each disallowed numeric literal in
/// `content`.
fn collect_magic_numbers(
    path: &Path,
    content: &str,
    magic_pattern: &Regex,
    violations: &mut Vec<OrganizationViolation>,
) {
    crate::validators::for_each_non_test_non_comment_line(content, |line_num, line, trimmed| {
        if skip_magic_line(trimmed) {
            return;
        }
        for cap in magic_pattern.captures_iter(line) {
            let num = cap.get(1).map_or("", |m| m.as_str());
            if is_allowed_magic(num, line) {
                continue;
            }
            violations.push(OrganizationViolation::MagicNumber {
                file: path.to_path_buf(),
                line: line_num + 1,
                value: num.to_owned(),
                context: trimmed.to_owned(),
                suggestion: "Consider using a named constant".to_owned(),
                severity: Severity::Info,
            });
        }
    });
}
