//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::constants::common::{COMMENT_PREFIX, FN_PREFIX, MODULE_DOC_PREFIX};
use crate::filters::LanguageId;
use crate::pattern_registry::{compile_regex, compile_regex_pairs};
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;

use super::violation::HygieneViolation;

struct QualityPatterns {
    mock_type: Regex,
    skip_message: Regex,
    todo: Regex,
    unimplemented: Regex,
}

struct QualityScanInput<'a> {
    test_attr_pattern: &'a Regex,
    fn_pattern: &'a Regex,
    real_assert_pattern: &'a Regex,
    unwrap_pattern: &'a Regex,
    compiled_trivial: &'a [(Regex, &'a str)],
}

struct TestBodyInput<'a> {
    path: &'a std::path::Path,
    lines: &'a [&'a str],
    fn_line_idx: usize,
    fn_name: &'a str,
    real_assert_pattern: &'a Regex,
    unwrap_pattern: &'a Regex,
    compiled_trivial: &'a [(Regex, &'a str)],
}

/// Validates test quality by checking for trivial assertions, unwrap-only tests, and comment-only tests.
///
/// # Errors
///
/// Returns an error if regex compilation, directory enumeration, or file reading fails.
pub fn validate_test_quality(config: &ValidationConfig) -> Result<Vec<HygieneViolation>> {
    let mut violations = Vec::new();

    let patterns = QualityPatterns {
        mock_type: compile_regex(r"\bMock[A-Za-z0-9_]+\b")?,
        skip_message: compile_regex("skipping:")?,
        todo: compile_regex(r"\btodo!\(")?,
        unimplemented: compile_regex(r"\bunimplemented!\(")?,
    };

    // Trivial assertion patterns
    let trivial_patterns = [
        (r"assert!\s*\(\s*true\s*\)", "assert!(true)"),
        (r"assert!\s*\(\s*!false\s*\)", "assert!(!false)"),
        (
            r"assert_eq!\s*\(\s*true\s*,\s*true\s*\)",
            "assert_eq!(true, true)",
        ),
        (r"assert_eq!\s*\(\s*1\s*,\s*1\s*\)", "assert_eq!(1, 1)"),
        (r"assert_ne!\s*\(\s*1\s*,\s*2\s*\)", "assert_ne!(1, 2)"),
        (
            r"assert_ne!\s*\(\s*true\s*,\s*false\s*\)",
            "assert_ne!(true, false)",
        ),
    ];

    let compiled_trivial = compile_regex_pairs(&trivial_patterns)?;

    let test_attr_pattern = compile_regex(r"#\[(?:tokio::)?test\]")?;
    let fn_pattern = compile_regex(r"(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")?;
    // Match common assertion macros - allow leading whitespace for indented code
    // The pattern checks for assertions at the start of a line (with optional whitespace)
    // or preceded by whitespace/punctuation to avoid false positives like "some_assert!"
    let real_assert_pattern =
        compile_regex(r"(?:^|\s)(assert[a-z_]*!|assert_[a-z_]+\(|debug_assert[a-z_]*!|panic!)")?;
    let unwrap_pattern = compile_regex(r"\.unwrap\(|\.expect\(")?;
    let scan_input = QualityScanInput {
        test_attr_pattern: &test_attr_pattern,
        fn_pattern: &fn_pattern,
        real_assert_pattern: &real_assert_pattern,
        unwrap_pattern: &unwrap_pattern,
        compiled_trivial: &compiled_trivial,
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

            violations.extend(process_quality_file(path, &lines, &patterns, &scan_input));
            Ok(())
        })?;
    }

    Ok(violations)
}

fn find_next_test_fn(
    lines: &[&str],
    start_idx: usize,
    fn_pattern: &Regex,
) -> Option<(usize, String)> {
    lines
        .iter()
        .enumerate()
        .skip(start_idx)
        .find_map(|(line_idx, candidate)| {
            let captures = fn_pattern.captures(candidate)?;
            let fn_name = captures.get(1).map_or("", |m| m.as_str());
            Some((line_idx, fn_name.to_owned()))
        })
}

fn process_quality_file(
    path: &std::path::Path,
    lines: &[&str],
    patterns: &QualityPatterns,
    scan_input: &QualityScanInput<'_>,
) -> Vec<HygieneViolation> {
    let mut violations = Vec::new();
    check_forbidden_patterns(path, lines, patterns, &mut violations);

    for (line_idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with(MODULE_DOC_PREFIX)
            || !scan_input.test_attr_pattern.is_match(line)
        {
            continue;
        }

        let Some((fn_line_idx, fn_name)) =
            find_next_test_fn(lines, line_idx + 1, scan_input.fn_pattern)
        else {
            continue;
        };

        let input = TestBodyInput {
            path,
            lines,
            fn_line_idx,
            fn_name: &fn_name,
            real_assert_pattern: scan_input.real_assert_pattern,
            unwrap_pattern: scan_input.unwrap_pattern,
            compiled_trivial: scan_input.compiled_trivial,
        };
        violations.extend(analyze_test_function_body(&input));
    }

    violations
}

fn analyze_test_function_body(input: &TestBodyInput<'_>) -> Vec<HygieneViolation> {
    let Some((body_lines, _)) = crate::scan::extract_balanced_block(input.lines, input.fn_line_idx)
    else {
        return Vec::new();
    };

    let mut violations = Vec::new();
    let mut has_assertion = false;
    let mut has_unwrap = false;
    let mut has_code = false;

    for (offset, body_line) in body_lines.iter().enumerate() {
        let body_line_idx = input.fn_line_idx + offset;
        let trimmed = body_line.trim();
        if should_skip_body_line(trimmed) {
            continue;
        }

        has_code = true;
        if input.real_assert_pattern.is_match(trimmed) {
            has_assertion = true;
        }
        if input.unwrap_pattern.is_match(trimmed) {
            has_unwrap = true;
        }

        for (regex, desc) in input.compiled_trivial {
            if regex.is_match(trimmed) {
                violations.push(HygieneViolation::TrivialAssertion {
                    file: input.path.to_path_buf(),
                    line: body_line_idx + 1,
                    function_name: input.fn_name.to_owned(),
                    assertion: (*desc).to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    if !has_code {
        violations.push(HygieneViolation::CommentOnlyTest {
            file: input.path.to_path_buf(),
            line: input.fn_line_idx + 1,
            function_name: input.fn_name.to_owned(),
            severity: Severity::Warning,
        });
    } else if !has_assertion && has_unwrap {
        violations.push(HygieneViolation::UnwrapOnlyAssertion {
            file: input.path.to_path_buf(),
            line: input.fn_line_idx + 1,
            function_name: input.fn_name.to_owned(),
            severity: Severity::Warning,
        });
    }

    violations
}

fn should_skip_body_line(trimmed: &str) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with(COMMENT_PREFIX)
        || trimmed.starts_with(FN_PREFIX)
        || matches!(trimmed, "{" | "}")
}

/// Checks for forbidden patterns in test files.
/// Checks for forbidden patterns in test files.
fn check_forbidden_patterns(
    file: &std::path::Path,
    lines: &[&str],
    patterns: &QualityPatterns,
    violations: &mut Vec<HygieneViolation>,
) {
    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;

        for mat in patterns.mock_type.find_iter(line) {
            violations.push(HygieneViolation::MockTypeUsage {
                file: file.to_path_buf(),
                line: line_no,
                token: mat.as_str().to_owned(),
                severity: Severity::Error,
            });
        }

        if patterns.skip_message.is_match(line) {
            violations.push(HygieneViolation::SkipBranchUsage {
                file: file.to_path_buf(),
                line: line_no,
                severity: Severity::Error,
            });
        }

        if patterns.todo.is_match(line) {
            violations.push(HygieneViolation::StubMacroUsage {
                file: file.to_path_buf(),
                line: line_no,
                macro_name: "todo".to_owned(),
                severity: Severity::Error,
            });
        }

        if patterns.unimplemented.is_match(line) {
            violations.push(HygieneViolation::StubMacroUsage {
                file: file.to_path_buf(),
                line: line_no,
                macro_name: "unimplemented".to_owned(),
                severity: Severity::Error,
            });
        }
    }

    for idx in 0..lines.len().saturating_sub(2) {
        let current = lines[idx].trim();
        let next = lines[idx + 1].trim();
        let next2 = lines[idx + 2].trim();
        if current.starts_with("let Ok(") && next.starts_with("else") && next2.contains("return;") {
            violations.push(HygieneViolation::SkipBranchUsage {
                file: file.to_path_buf(),
                line: idx + 1,
                severity: Severity::Error,
            });
        }
    }
}
