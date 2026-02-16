use std::path::Path;
use std::path::PathBuf;

use regex::Regex;

use crate::Result;
use crate::constants::common::{CFG_TEST_MARKER, COMMENT_PREFIX, TEST_DIR_FRAGMENT};
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;

use super::PerformanceValidator;
use super::violation::PerformanceViolation;

fn should_skip_scan_file(validator: &PerformanceValidator, src_dir: &Path, path: &Path) -> bool {
    validator.should_skip_crate(src_dir)
        || path.to_str().is_some_and(|s| s.contains(TEST_DIR_FRAGMENT))
}

fn for_each_relevant_file_content<F>(validator: &PerformanceValidator, mut f: F) -> Result<()>
where
    F: FnMut(PathBuf, String) -> Result<()>,
{
    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, src_dir| {
            let path = &entry.absolute_path;
            if should_skip_scan_file(validator, src_dir, path) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            f(path.clone(), content)
        },
    )
}

fn apply_line_brace_delta(loop_depth: &mut i32, line: &str) {
    *loop_depth += line.chars().filter(|c| *c == '{').count() as i32;
    *loop_depth -= line.chars().filter(|c| *c == '}').count() as i32;
}

struct LoopPatternScanInput<'a> {
    path: &'a Path,
    content: &'a str,
    loop_start_pattern: &'a Regex,
    patterns: &'a [Regex],
}

fn collect_loop_pattern_violations<F>(
    input: &LoopPatternScanInput<'_>,
    make_violation: &F,
    violations: &mut Vec<PerformanceViolation>,
) where
    F: Fn(PathBuf, usize, &str) -> Option<PerformanceViolation>,
{
    let LoopPatternScanInput {
        path,
        content,
        loop_start_pattern,
        patterns,
    } = input;

    let mut in_loop = false;
    let mut loop_depth = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        if loop_start_pattern.is_match(trimmed) {
            in_loop = true;
            loop_depth = 0;
        }

        if !in_loop {
            continue;
        }

        apply_line_brace_delta(&mut loop_depth, line);
        for pattern in *patterns {
            if pattern.is_match(line)
                && let Some(violation) = make_violation(path.to_path_buf(), line_num + 1, line)
            {
                violations.push(violation);
            }
        }

        if loop_depth <= 0 {
            in_loop = false;
        }
    }
}

fn collect_pattern_violations<F>(
    path: &Path,
    content: &str,
    compiled_patterns: &[(Regex, &str, &str)],
    make_violation: &F,
    violations: &mut Vec<PerformanceViolation>,
) where
    F: Fn(PathBuf, usize, &str, &str) -> PerformanceViolation,
{
    let mut in_test_module = false;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

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

        for (pattern, desc, sugg) in compiled_patterns {
            if pattern.is_match(line) {
                violations.push(make_violation(path.to_path_buf(), line_num + 1, desc, sugg));
            }
        }
    }
}

/// Helper: Scan files for patterns inside loops
pub fn scan_files_with_patterns_in_loops<F>(
    validator: &PerformanceValidator,
    patterns: &[Regex],
    make_violation: F,
) -> Result<Vec<PerformanceViolation>>
where
    F: Fn(PathBuf, usize, &str) -> Option<PerformanceViolation>,
{
    let mut violations = Vec::new();
    let loop_start_pattern = compile_regex(r"^\s*(for|while|loop)\s+")?;

    for_each_relevant_file_content(validator, |path, content| {
        collect_loop_pattern_violations(
            &LoopPatternScanInput {
                path: &path,
                content: &content,
                loop_start_pattern: &loop_start_pattern,
                patterns,
            },
            &make_violation,
            &mut violations,
        );
        Ok(())
    })?;

    Ok(violations)
}

/// Helper: Scan files and apply pattern matching with a custom violation builder.
pub fn scan_files_with_patterns<F>(
    validator: &PerformanceValidator,
    compiled_patterns: &[(Regex, &str, &str)],
    make_violation: F,
) -> Result<Vec<PerformanceViolation>>
where
    F: Fn(PathBuf, usize, &str, &str) -> PerformanceViolation,
{
    let mut violations = Vec::new();

    for_each_relevant_file_content(validator, |path, content| {
        collect_pattern_violations(
            &path,
            &content,
            compiled_patterns,
            &make_violation,
            &mut violations,
        );

        Ok(())
    })?;

    Ok(violations)
}
