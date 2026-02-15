use std::sync::LazyLock;

use regex::Regex;

use super::constants::PANIC_REGEX;
use super::{QualityValidator, QualityViolation};
use crate::constants::common::{CFG_TEST_MARKER, COMMENT_PREFIX};
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

static PANIC_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PANIC_REGEX).expect("valid regex literal"));

/// Scans production code for usage of the `panic!()` macro.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, _src_dir| {
            if entry
                .absolute_path
                .extension()
                .is_none_or(|ext| ext != "rs")
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with(COMMENT_PREFIX) {
                    continue;
                }

                if trimmed.contains(CFG_TEST_MARKER) {
                    in_test_module = true;
                    continue;
                }

                if in_test_module {
                    continue;
                }

                // Check for panic!
                if PANIC_PATTERN.is_match(line) {
                    violations.push(QualityViolation::PanicInProduction {
                        file: entry.absolute_path.clone(),
                        line: line_num + 1,
                        context: trimmed.to_owned(),
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
