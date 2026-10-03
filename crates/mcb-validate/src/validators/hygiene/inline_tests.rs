//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use regex::Regex;

use crate::pattern_registry::required_pattern;
use crate::utils::source::for_each_rust_file;
use crate::{Result, Severity, ValidationConfig};
use mcb_utils::constants::validate::COMMENT_PREFIX;

use super::violation::HygieneViolation;

/// Regexes that mark inline test declarations within a source file.
struct InlineTestPatterns {
    cfg_test: &'static Regex,
    mod_tests: &'static Regex,
}

/// Verifies that no inline test declarations exist in src/ directories.
///
/// # Errors
///
/// Returns an error if pattern loading, directory enumeration, or file reading fails.
pub fn validate_no_inline_tests(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let patterns = InlineTestPatterns {
        cfg_test: required_pattern("TEST001.cfg_test")?,
        mod_tests: required_pattern("TEST001.mod_tests")?,
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
    let (mut flagged, has_inline_module_marker) =
        collect_inline_module_marker_lines(lines, patterns);

    if !has_inline_module_marker && let Some(line) = first_test_attribute_line(lines) {
        flagged.push(line);
    }

    flagged
}

fn collect_inline_module_marker_lines(
    lines: &[&str],
    patterns: &InlineTestPatterns,
) -> (Vec<usize>, bool) {
    let mut flagged = Vec::new();
    let mut last_cfg_test_line: Option<usize> = None;
    let mut has_inline_module_marker = false;

    for (line_num, line) in lines.iter().enumerate() {
        if !is_inline_test_scan_line(line) {
            continue;
        }
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

    (flagged, has_inline_module_marker)
}

fn first_test_attribute_line(lines: &[&str]) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .find(|(_, line)| is_test_attribute_line(line))
        .map(|(line_num, _)| line_num + 1)
}

fn is_inline_test_scan_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.starts_with(COMMENT_PREFIX) && !trimmed.starts_with('"')
}

fn is_test_attribute_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "#[test]" || trimmed == "#[tokio::test]"
}
