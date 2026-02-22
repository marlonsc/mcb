//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
use super::constants::PANIC_REGEX;
use super::{QualityValidator, QualityViolation};
use crate::constants::common::{CFG_TEST_MARKER, COMMENT_PREFIX};
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

/// Scans production code for usage of the `panic!()` macro.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    let panic_pattern = compile_regex(PANIC_REGEX)?;

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

                in_test_module = in_test_module || trimmed.contains(CFG_TEST_MARKER);
                if !trimmed.starts_with(COMMENT_PREFIX)
                    && !in_test_module
                    && panic_pattern.is_match(line)
                {
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
