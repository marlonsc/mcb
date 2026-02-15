use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regex_triples, required_pattern};
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

use super::violation::AsyncViolation;

/// Detect `std::sync::Mutex` usage in files with async code
pub fn validate_mutex_types(config: &ValidationConfig) -> Result<Vec<AsyncViolation>> {
    let mut violations = Vec::new();

    let async_indicator = required_pattern("ASYNC001.async_indicator")?;
    let std_mutex_patterns = [
        (
            r"use\s+std::sync::Mutex",
            "std::sync::Mutex import",
            "Use tokio::sync::Mutex for async code",
        ),
        (
            "std::sync::Mutex<",
            "std::sync::Mutex type",
            "Use tokio::sync::Mutex for async code",
        ),
        (
            r"use\s+std::sync::RwLock",
            "std::sync::RwLock import",
            "Use tokio::sync::RwLock for async code",
        ),
        (
            "std::sync::RwLock<",
            "std::sync::RwLock type",
            "Use tokio::sync::RwLock for async code",
        ),
    ];

    let compiled_mutex = compile_regex_triples(&std_mutex_patterns)?;

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
        let path = &entry.absolute_path;
        if path.to_str().is_some_and(|s| s.contains("/tests/")) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;

        // Only check files that have async code
        if !async_indicator.is_match(&content) {
            return Ok(());
        }

        let mut in_test_module = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            // Track test modules
            if trimmed.contains("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            if in_test_module {
                continue;
            }

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
        }

        Ok(())
    })?;

    Ok(violations)
}
