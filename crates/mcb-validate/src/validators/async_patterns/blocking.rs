use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regex_triples, required_pattern};
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

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

                // Check for blocking calls
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

                if async_fn_depth <= 0 {
                    in_async_fn = false;
                }
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
