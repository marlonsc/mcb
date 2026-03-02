//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
use super::violation::OrganizationViolation;
use crate::constants::common::{
    ATTRIBUTE_PREFIX, CONST_DECLARATION_PREFIXES, CONSTANTS_FILE_KEYWORDS,
};
use crate::constants::organization::{
    DUPLICATE_STRING_MIN_FILES, DUPLICATE_STRING_REGEX, DUPLICATE_STRING_SKIP_PATTERNS,
};
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::{for_each_crate_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Scans for string literals duplicated across multiple files that should be centralized.
///
/// # Errors
///
/// Returns an error if file scanning or reading fails.
pub fn validate_duplicate_strings(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();
    let mut string_occurrences: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

    // Pattern for string literals (15+ chars to reduce noise)
    let string_pattern = compile_regex(DUPLICATE_STRING_REGEX)?;

    for_each_crate_file(
        config,
        Some(LanguageId::Rust),
        |entry, _src_dir, _crate_name| {
            let path = &entry.absolute_path;
            // Skip constants files (they define string constants)
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
                    let skip_line = trimmed.starts_with(ATTRIBUTE_PREFIX)
                        || CONST_DECLARATION_PREFIXES
                            .iter()
                            .any(|p| trimmed.starts_with(p));
                    if skip_line {
                        return;
                    }

                    string_occurrences.extend(
                        string_pattern
                            .captures_iter(line)
                            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_owned()))
                            .filter(|s| {
                                !DUPLICATE_STRING_SKIP_PATTERNS
                                    .iter()
                                    .any(|pat| s.contains(pat))
                            })
                            .map(|string_val| (string_val, vec![(path.clone(), line_num + 1)])),
                    );
                },
            );

            Ok(())
        },
    )?;

    // Report strings that appear in 4+ files (higher threshold)
    violations.extend(
        string_occurrences
            .into_iter()
            .filter_map(|(value, occurrences)| {
                let unique_files: HashSet<_> = occurrences.iter().map(|(f, _)| f).collect();
                (unique_files.len() >= DUPLICATE_STRING_MIN_FILES).then(|| {
                    OrganizationViolation::DuplicateStringLiteral {
                        value,
                        occurrences,
                        suggestion: "Consider creating a named constant".to_owned(),
                        severity: Severity::Info,
                    }
                })
            }),
    );

    Ok(violations)
}
