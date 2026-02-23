//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::pattern_registry::{required_pattern, required_patterns};
use crate::utils::source::for_each_rust_file;
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

    for_each_rust_file(config, |path, lines| {
        if path.to_str().is_some_and(|s| s.contains("/fixtures/")) {
            return Ok(());
        }

        let mut push_violation = |line: usize| {
            violations.push(HygieneViolation::InlineTestModule {
                file: path.clone(),
                line,
                severity: Severity::Warning,
            });
        };

        let mut last_cfg_test_line: Option<usize> = None;
        let mut has_inline_module_marker = false;

        for (line_num, line) in lines.iter().enumerate() {
            let has_recent_cfg =
                last_cfg_test_line.is_some_and(|cfg_line| line_num <= cfg_line + 5);
            let is_cfg_test = cfg_test_pattern.is_match(line);
            let is_orphan_test_mod = mod_tests_pattern.is_match(line) && !has_recent_cfg;

            if is_cfg_test || is_orphan_test_mod {
                if is_cfg_test {
                    last_cfg_test_line = Some(line_num);
                }
                has_inline_module_marker = true;
                push_violation(line_num + 1);
            }
        }

        if !has_inline_module_marker
            && let Some((line_num, _)) = lines.iter().enumerate().find(|(_, line)| {
                test_attr_pattern.is_match(line) || tokio_test_attr_pattern.is_match(line)
            })
        {
            push_violation(line_num + 1);
        }

        Ok(())
    })?;

    Ok(violations)
}
