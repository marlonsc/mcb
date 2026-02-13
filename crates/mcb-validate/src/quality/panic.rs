use regex::Regex;

use super::{QualityValidator, QualityViolation};
use crate::scan::for_each_scan_rs_path;
use crate::{Result, Severity};

/// Scans production code for usage of the `panic!()` macro.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    // # Code Quality Violation (QUAL001)
    // Static regex initialization using .unwrap() is risky in production.
    //
    // TODO(QUAL001): Use LazyLock or proper error handling for Regex creation.
    let panic_pattern = Regex::new(r"panic!\s*\(").unwrap();

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

            // # False Positive (TEST001)
            // This line contains a check for #[cfg(test)], which is interpreted as an inline test module.
            //
            // TODO(TEST001): Refactor detection logic to distinguish between test code and test-detection code.
            if trimmed.contains("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            if in_test_module {
                continue;
            }

            // Check for panic!
            if panic_pattern.is_match(line) {
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
