use super::constants::{ALLOWED_MAGIC_NUMBERS, MAGIC_NUMBER_REGEX};
use super::violation::OrganizationViolation;
use crate::constants::common::{
    ATTRIBUTE_PREFIX, CFG_TEST_MARKER, COMMENT_PREFIX, CONST_DECLARATION_PREFIXES,
    CONSTANTS_FILE_KEYWORDS, DOC_COMMENT_PREFIX, MODULE_DOC_PREFIX,
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
            let Some(path_str) = path.to_str() else {
                return Ok(());
            };
            if is_test_path(path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with(COMMENT_PREFIX) {
                    continue;
                }

                // Track test module context
                if trimmed.contains(CFG_TEST_MARKER) {
                    in_test_module = true;
                    continue;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip const/static definitions (they're creating constants)
                if CONST_DECLARATION_PREFIXES
                    .iter()
                    .any(|p| trimmed.starts_with(p))
                {
                    continue;
                }

                // Skip attribute macros (derive, cfg, etc.)
                if trimmed.starts_with(ATTRIBUTE_PREFIX) {
                    continue;
                }

                // Skip doc comments
                if trimmed.starts_with(DOC_COMMENT_PREFIX) || trimmed.starts_with(MODULE_DOC_PREFIX)
                {
                    continue;
                }

                // Skip assert macros (often use expected values)
                if trimmed.contains("assert") {
                    continue;
                }

                for cap in magic_pattern.captures_iter(line) {
                    let num = cap.get(1).map_or("", |m| m.as_str());

                    // Skip allowed numbers
                    if ALLOWED_MAGIC_NUMBERS.contains(&num) {
                        continue;
                    }

                    // Skip numbers that are clearly part of a constant reference
                    // e.g., _1024, SIZE_16384
                    if line.contains(&format!("_{num}")) || line.contains(&format!("{num}_")) {
                        continue;
                    }

                    // Skip underscored numbers (100_000) - they're usually constants
                    if line.contains(&format!(
                        "{}_{}",
                        &num[..num.len().min(3)],
                        &num[num.len().min(3)..]
                    )) {
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
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
