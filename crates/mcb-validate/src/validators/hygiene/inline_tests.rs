use crate::filters::LanguageId;
use crate::pattern_registry::{required_pattern, required_patterns};
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;

/// Verifies that no inline test declarations exist in src/ directories.
///
/// # Errors
///
/// Returns an error if pattern loading, directory enumeration, or file reading fails.
pub fn validate_no_inline_tests(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let cfg_test_pattern = required_pattern("TEST001.cfg_test")?;
    let mod_tests_pattern = required_pattern("TEST001.mod_tests")?;
    let [test_attr_pattern, tokio_test_attr_pattern] =
        required_patterns(["TEST001.test_attr", "TEST001.tokio_test_attr"])?
            .try_into()
            .map_err(|_| crate::ValidationError::Config("invalid test pattern set".to_owned()))?;

    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        // We use is_fixture_path logic locally or duplicated?
        // Original code: if Self::is_fixture_path(path) { return Ok(()); }
        // Let's implement a local helper or use a shared one if available.
        // Similar to organization/validator.rs using generic scan, we can filter inside.

        for_each_file_under_root(config, &src_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            if path.to_str().is_some_and(|s| s.contains("/fixtures/")) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();
            let mut last_cfg_test_line: Option<usize> = None;
            let mut has_inline_module_marker = false;

            for (line_num, line) in lines.iter().enumerate() {
                if cfg_test_pattern.is_match(line) {
                    last_cfg_test_line = Some(line_num);
                    has_inline_module_marker = true;
                    violations.push(HygieneViolation::InlineTestModule {
                        file: path.clone(),
                        line: line_num + 1,
                        severity: Severity::Warning,
                    });
                    continue;
                }

                if mod_tests_pattern.is_match(line) {
                    if last_cfg_test_line.is_some_and(|cfg_line| line_num <= cfg_line + 5) {
                        continue;
                    }
                    has_inline_module_marker = true;
                    violations.push(HygieneViolation::InlineTestModule {
                        file: path.clone(),
                        line: line_num + 1,
                        severity: Severity::Warning,
                    });
                }
            }

            if !has_inline_module_marker {
                for (line_num, line) in lines.iter().enumerate() {
                    if test_attr_pattern.is_match(line) || tokio_test_attr_pattern.is_match(line) {
                        violations.push(HygieneViolation::InlineTestModule {
                            file: path.clone(),
                            line: line_num + 1,
                            severity: Severity::Warning,
                        });
                        break;
                    }
                }
            }
            Ok(())
        })?;
    }

    Ok(violations)
}
