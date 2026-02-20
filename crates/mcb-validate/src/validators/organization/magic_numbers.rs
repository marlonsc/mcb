//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
use super::constants::{ALLOWED_MAGIC_NUMBERS, MAGIC_NUMBER_REGEX};
use super::violation::OrganizationViolation;
use crate::constants::common::{
    ATTRIBUTE_PREFIX, CONST_DECLARATION_PREFIXES, CONSTANTS_FILE_KEYWORDS, DOC_COMMENT_PREFIX,
    MODULE_DOC_PREFIX,
};
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::{for_each_crate_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};

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
            crate::validators::for_each_non_test_non_comment_line(
                &content,
                |line_num, line, trimmed| {
                    let skip_line = CONST_DECLARATION_PREFIXES
                        .iter()
                        .any(|p| trimmed.starts_with(p))
                        || [ATTRIBUTE_PREFIX, DOC_COMMENT_PREFIX, MODULE_DOC_PREFIX]
                            .iter()
                            .any(|prefix| trimmed.starts_with(prefix))
                        || trimmed.contains("assert");
                    if skip_line {
                        return;
                    }

                    for cap in magic_pattern.captures_iter(line) {
                        let num = cap.get(1).map_or("", |m| m.as_str());
                        let segmented =
                            format!("{}_{}", &num[..num.len().min(3)], &num[num.len().min(3)..]);

                        if ALLOWED_MAGIC_NUMBERS.contains(&num)
                            || line.contains(&format!("_{num}"))
                            || line.contains(&format!("{num}_"))
                            || line.contains(&segmented)
                        {
                            continue;
                        }

                        violations.push(OrganizationViolation::MagicNumber {
                            file: path.clone(),
                            line: line_num + 1,
                            value: num.to_owned(),
                            context: trimmed.to_owned(),
                            suggestion: "Consider using a named constant".to_owned(),
                            severity: Severity::Info,
                        });
                    }
                },
            );

            Ok(())
        },
    )?;

    Ok(violations)
}
