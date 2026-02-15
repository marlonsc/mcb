use super::constants::{
    DUPLICATE_STRING_MIN_FILES, DUPLICATE_STRING_REGEX, DUPLICATE_STRING_SKIP_PATTERNS,
};
use super::violation::OrganizationViolation;
use crate::constants::common::{
    ATTRIBUTE_PREFIX, CFG_TEST_MARKER, COMMENT_PREFIX, CONST_DECLARATION_PREFIXES,
    CONSTANTS_FILE_KEYWORDS,
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

                // Skip comments and doc strings
                if trimmed.starts_with(COMMENT_PREFIX) || trimmed.starts_with(ATTRIBUTE_PREFIX) {
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

                // Skip const/static definitions
                if CONST_DECLARATION_PREFIXES
                    .iter()
                    .any(|p| trimmed.starts_with(p))
                {
                    continue;
                }

                for cap in string_pattern.captures_iter(line) {
                    let string_val = cap.get(1).map_or("", |m| m.as_str());

                    // Skip common patterns that are OK to repeat
                    if DUPLICATE_STRING_SKIP_PATTERNS
                        .iter()
                        .any(|pat| string_val.contains(pat))
                    {
                        continue;
                    }

                    string_occurrences
                        .entry(string_val.to_owned())
                        .or_default()
                        .push((path.clone(), line_num + 1));
                }
            }

            Ok(())
        },
    )?;

    // Report strings that appear in 4+ files (higher threshold)
    for (value, occurrences) in string_occurrences {
        let unique_files: HashSet<_> = occurrences.iter().map(|(f, _)| f).collect();
        if unique_files.len() >= DUPLICATE_STRING_MIN_FILES {
            violations.push(OrganizationViolation::DuplicateStringLiteral {
                value,
                occurrences,
                suggestion: "Consider creating a named constant".to_owned(),
                severity: Severity::Info,
            });
        }
    }

    Ok(violations)
}
