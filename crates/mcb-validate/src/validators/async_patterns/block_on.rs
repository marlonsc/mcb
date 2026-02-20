//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::constants::common::{CONTEXT_PREVIEW_LENGTH, TEST_DIR_FRAGMENT};
use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regexes, required_pattern};
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

use super::for_each_async_fn_line;
use super::violation::AsyncViolation;

/// Detect `block_on()` usage in async context
pub fn validate_block_on_usage(config: &ValidationConfig) -> Result<Vec<AsyncViolation>> {
    let mut violations = Vec::new();

    let async_fn_pattern = required_pattern("ASYNC001.async_fn")?;
    let block_on_patterns = [
        r"block_on\(",
        "futures::executor::block_on",
        r"tokio::runtime::Runtime::new\(\).*\.block_on",
        r"Runtime::new\(\).*\.block_on",
    ];

    let compiled_block_on = compile_regexes(block_on_patterns)?;

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
        let path = &entry.absolute_path;
        if path.to_str().is_some_and(|s| s.contains(TEST_DIR_FRAGMENT)) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;

        for_each_async_fn_line(&content, async_fn_pattern, |line_num, line, trimmed| {
            for pattern in &compiled_block_on {
                if pattern.is_match(line) {
                    violations.push(AsyncViolation::BlockOnInAsync {
                        file: path.clone(),
                        line: line_num + 1,
                        context: trimmed.chars().take(CONTEXT_PREVIEW_LENGTH).collect(),
                        severity: Severity::Error,
                    });
                }
            }
        });

        Ok(())
    })?;

    Ok(violations)
}
