//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::constants::common::TEST_DIR_FRAGMENT;
use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regex_triples, required_pattern};
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

use super::for_each_async_fn_line;
use super::violation::AsyncViolation;

/// Detect blocking calls in async functions
pub fn validate_blocking_in_async(config: &ValidationConfig) -> Result<Vec<AsyncViolation>> {
    let mut violations = Vec::new();

    let async_fn_pattern = required_pattern("ASYNC001.async_fn_named")?;

    let blocking_patterns = [
        (
            "std::thread::sleep",
            "std::thread::sleep",
            "Use tokio::time::sleep instead",
        ),
        (
            "thread::sleep",
            "thread::sleep",
            "Use tokio::time::sleep instead",
        ),
        (
            "std::fs::read",
            "std::fs::read",
            "Use tokio::fs::read instead",
        ),
        (
            "std::fs::write",
            "std::fs::write",
            "Use tokio::fs::write instead",
        ),
        (
            "std::fs::File::open",
            "std::fs::File::open",
            "Use tokio::fs::File::open instead",
        ),
        (
            "std::fs::File::create",
            "std::fs::File::create",
            "Use tokio::fs::File::create instead",
        ),
        (
            r"\.blocking_lock\(\)",
            ".blocking_lock()",
            "Use .lock().await instead in async context",
        ),
        (
            r"\.blocking_read\(\)",
            ".blocking_read()",
            "Use .read().await instead in async context",
        ),
        (
            r"\.blocking_write\(\)",
            ".blocking_write()",
            "Use .write().await instead in async context",
        ),
    ];

    let compiled_blocking = compile_regex_triples(&blocking_patterns)?;

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
        let path = &entry.absolute_path;
        if path.to_str().is_some_and(|s| s.contains(TEST_DIR_FRAGMENT)) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;

        for_each_async_fn_line(&content, async_fn_pattern, |line_num, line, _trimmed| {
            for (pattern, desc, sugg) in &compiled_blocking {
                if pattern.is_match(line) {
                    violations.push(AsyncViolation::BlockingInAsync {
                        file: path.clone(),
                        line: line_num + 1,
                        blocking_call: desc.to_string(),
                        suggestion: sugg.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        });

        Ok(())
    })?;

    Ok(violations)
}
