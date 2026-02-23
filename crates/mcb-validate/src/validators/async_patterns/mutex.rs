//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use super::constants::WRONG_MUTEX_PATTERNS;
use crate::constants::common::TEST_PATH_PATTERNS;
use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regex_triples, required_pattern};
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

use super::violation::AsyncViolation;

/// Detect `std::sync::Mutex` usage in files with async code
pub fn validate_mutex_types(config: &ValidationConfig) -> Result<Vec<AsyncViolation>> {
    let mut violations = Vec::new();

    let async_indicator = required_pattern("ASYNC001.async_indicator")?;
    let compiled_mutex = compile_regex_triples(WRONG_MUTEX_PATTERNS)?;

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
        let path = &entry.absolute_path;
        if path
            .to_str()
            .is_some_and(|s| TEST_PATH_PATTERNS.iter().any(|p| s.contains(p)))
        {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;

        // Only check files that have async code
        if !async_indicator.is_match(&content) {
            return Ok(());
        }

        crate::validators::for_each_non_test_non_comment_line(
            &content,
            |line_num, line, _trimmed| {
                // Check for std mutex patterns
                for (pattern, desc, sugg) in &compiled_mutex {
                    if pattern.is_match(line) {
                        violations.push(AsyncViolation::WrongMutexType {
                            file: path.clone(),
                            line: line_num + 1,
                            mutex_type: desc.to_string(),
                            suggestion: sugg.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            },
        );

        Ok(())
    })?;

    Ok(violations)
}
