use crate::constants::common::CONTEXT_PREVIEW_LENGTH;
use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regexes, required_pattern};
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

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
        if path.to_str().is_some_and(|s| s.contains("/tests/")) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut in_async_fn = false;
        let mut async_fn_depth = 0;
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

            // Track async function entry
            if async_fn_pattern.is_match(trimmed) {
                in_async_fn = true;
                async_fn_depth = 0;
            }

            if in_async_fn {
                let open = line.chars().filter(|c| *c == '{').count();
                let close = line.chars().filter(|c| *c == '}').count();
                async_fn_depth += i32::try_from(open).unwrap_or(i32::MAX);
                async_fn_depth -= i32::try_from(close).unwrap_or(i32::MAX);

                // Check for block_on calls
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

                if async_fn_depth <= 0 {
                    in_async_fn = false;
                }
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
