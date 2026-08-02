//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use regex::Regex;

use crate::pattern_registry::{required_pattern, required_patterns};
use crate::utils::source::for_each_rust_file;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;

/// Regexes that mark inline test declarations within a source file.
struct InlineTestPatterns {
    cfg_test: &'static Regex,
    mod_tests: &'static Regex,
    test_attr: &'static Regex,
    tokio_test_attr: &'static Regex,
}

/// Verifies that no inline test declarations exist in src/ directories.
///
/// # Errors
///
/// Returns an error if pattern loading, directory enumeration, or file reading fails.
pub fn validate_no_inline_tests(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let [test_attr, tokio_test_attr] =
        required_patterns(["TEST001.test_attr", "TEST001.tokio_test_attr"])?
            .try_into()
            .map_err(|_| crate::ValidationError::Config("invalid test pattern set".to_owned()))?;
    let patterns = InlineTestPatterns {
        cfg_test: required_pattern("TEST001.cfg_test")?,
        mod_tests: required_pattern("TEST001.mod_tests")?,
        test_attr,
        tokio_test_attr,
    };

    for_each_rust_file(config, |path, lines| {
        if path.to_str().is_some_and(|s| s.contains("/fixtures/")) {
            return Ok(());
        }
        for line in collect_inline_test_lines(&lines, &patterns) {
            violations.push(HygieneViolation::InlineTestModule {
                file: path.clone(),
                line,
                severity: Severity::Warning,
            });
        }
        Ok(())
    })?;

    Ok(violations)
}

/// Collect the 1-based line numbers of inline test markers in `lines`. Falls
/// back to the first `#[test]`/`#[tokio::test]` attribute when no module marker
/// is present.
fn collect_inline_test_lines(lines: &[&str], patterns: &InlineTestPatterns) -> Vec<usize> {
    let mut flagged = Vec::new();
    let mut last_cfg_test_line: Option<usize> = None;
    let mut has_inline_module_marker = false;

    for (line_num, line) in lines.iter().enumerate() {
        let has_recent_cfg = last_cfg_test_line.is_some_and(|cfg_line| line_num <= cfg_line + 5);
        let is_cfg_test = patterns.cfg_test.is_match(line);
        let is_orphan_test_mod = patterns.mod_tests.is_match(line) && !has_recent_cfg;

        if is_cfg_test || is_orphan_test_mod {
            if is_cfg_test {
                last_cfg_test_line = Some(line_num);
            }
            has_inline_module_marker = true;
            flagged.push(line_num + 1);
        }
    }

    if !has_inline_module_marker
        && let Some((line_num, _)) = lines.iter().enumerate().find(|(_, line)| {
            patterns.test_attr.is_match(line) || patterns.tokio_test_attr.is_match(line)
        })
    {
        flagged.push(line_num + 1);
    }

    flagged
}
