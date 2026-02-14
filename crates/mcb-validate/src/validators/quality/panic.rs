use std::sync::LazyLock;

use regex::Regex;

use super::{QualityValidator, QualityViolation};
use crate::scan::for_each_scan_rs_path;
use crate::{Result, Severity};

static PANIC_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"panic!\s*\(").expect("valid regex literal"));

/// Scans production code for usage of the `panic!()` macro.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    for_each_scan_rs_path(&validator.config, false, |path, _src_dir| {
        if path.extension().is_none_or(|ext| ext != "rs") {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut in_test_module = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            if trimmed.contains("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            if in_test_module {
                continue;
            }

            // Check for panic!
            if PANIC_PATTERN.is_match(line) {
                violations.push(QualityViolation::PanicInProduction {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    context: trimmed.to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
