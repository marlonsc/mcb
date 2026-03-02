//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity, ValidationConfig};

use super::violation::HygieneViolation;
use crate::ValidationConfigExt;

const SMOKE_TEST_PATTERNS: [&str; 5] = [
    "_trait_object",
    "_exists",
    "_creation",
    "_compiles",
    "_factory",
];

struct NamingScanInput<'a> {
    test_attr_pattern: &'a regex::Regex,
    tokio_test_pattern: &'a regex::Regex,
    fn_pattern: &'a regex::Regex,
    assert_pattern: &'a regex::Regex,
}

/// Verifies that test functions follow the `test_*` naming pattern.
///
/// # Errors
///
/// Returns an error if source directory enumeration or file reading fails.
pub fn validate_test_function_naming(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();
    let test_attr_pattern = compile_regex(r"#\[test\]")?;
    let tokio_test_pattern = compile_regex(r"#\[tokio::test\]")?;
    let fn_pattern = compile_regex(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")?;
    let assert_pattern = compile_regex(
        r"assert[a-z_]*!|assert_[a-z_]+\(|panic!|should_panic|\.unwrap\(|\.expect\(|Box<dyn\s|type_name::",
    )?;
    let scan_input = NamingScanInput {
        test_attr_pattern: &test_attr_pattern,
        tokio_test_pattern: &tokio_test_pattern,
        fn_pattern: &fn_pattern,
        assert_pattern: &assert_pattern,
    };

    for crate_dir in config.get_source_dirs()? {
        let tests_dir = crate_dir.join("tests");
        if !tests_dir.exists() {
            continue;
        }

        for_each_file_under_root(config, &tests_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            if path.to_str().is_some_and(|s| s.contains("/fixtures/")) {
                return Ok(());
            }
            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            violations.extend(collect_naming_violations_for_file(
                path,
                &lines,
                &scan_input,
            ));
            Ok(())
        })?;
    }

    Ok(violations)
}

fn is_test_attr_line(
    line: &str,
    test_attr_pattern: &regex::Regex,
    tokio_test_pattern: &regex::Regex,
) -> bool {
    test_attr_pattern.is_match(line) || tokio_test_pattern.is_match(line)
}

fn find_next_test_function(
    lines: &[&str],
    start_idx: usize,
    fn_pattern: &regex::Regex,
) -> Option<(usize, String)> {
    lines
        .iter()
        .enumerate()
        .skip(start_idx)
        .find_map(|(line_idx, potential_fn)| {
            let captures = fn_pattern.captures(potential_fn)?;
            let fn_name = captures.get(1).map_or("", |m| m.as_str());
            Some((line_idx, fn_name.to_owned()))
        })
}

fn append_naming_violation_if_needed(
    violations: &mut Vec<HygieneViolation>,
    path: &std::path::Path,
    fn_line_idx: usize,
    fn_name: &str,
) {
    if fn_name.starts_with("test_") {
        return;
    }

    violations.push(HygieneViolation::BadTestFunctionName {
        file: path.to_path_buf(),
        line: fn_line_idx + 1,
        function_name: fn_name.to_owned(),
        suggestion: format!("test_{fn_name}"),
        severity: Severity::Warning,
    });
}

fn has_assertion_in_test_body(
    lines: &[&str],
    fn_line_idx: usize,
    assert_pattern: &regex::Regex,
) -> bool {
    let Some((body_lines, _)) = crate::scan::extract_balanced_block(lines, fn_line_idx) else {
        return false;
    };

    body_lines.iter().any(|line| assert_pattern.is_match(line))
}

fn is_smoke_test(fn_name: &str) -> bool {
    SMOKE_TEST_PATTERNS
        .iter()
        .any(|pattern| fn_name.ends_with(pattern))
}

fn collect_naming_violations_for_file(
    path: &std::path::Path,
    lines: &[&str],
    scan_input: &NamingScanInput<'_>,
) -> Vec<HygieneViolation> {
    let mut violations = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        if !is_test_attr_line(
            line,
            scan_input.test_attr_pattern,
            scan_input.tokio_test_pattern,
        ) {
            continue;
        }

        let Some((fn_line_idx, fn_name)) =
            find_next_test_function(lines, line_idx + 1, scan_input.fn_pattern)
        else {
            continue;
        };

        append_naming_violation_if_needed(&mut violations, path, fn_line_idx, &fn_name);

        let has_assertion =
            has_assertion_in_test_body(lines, fn_line_idx, scan_input.assert_pattern);
        if !has_assertion && !is_smoke_test(&fn_name) {
            violations.push(HygieneViolation::TestWithoutAssertion {
                file: path.to_path_buf(),
                line: fn_line_idx + 1,
                function_name: fn_name,
                severity: Severity::Warning,
            });
        }
    }

    violations
}
