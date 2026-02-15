use std::path::PathBuf;

use regex::Regex;

use crate::Result;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;

use super::PerformanceValidator;
use super::violation::PerformanceViolation;

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
    let loop_start_pattern = Regex::new(r"^\s*(for|while|loop)\s+")?;

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, src_dir| {
            let path = &entry.absolute_path;
            if validator.should_skip_crate(src_dir)
                || path.to_str().is_some_and(|s| s.contains("/tests/"))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_loop = false;
            let mut loop_depth = 0;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                if trimmed.starts_with("//") {
                    continue;
                }

                if loop_start_pattern.is_match(trimmed) {
                    in_loop = true;
                    loop_depth = 0;
                }

                if in_loop {
                    loop_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    loop_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    for pattern in patterns {
                        if pattern.is_match(line)
                            && let Some(violation) =
                                make_violation(path.to_path_buf(), line_num + 1, line)
                        {
                            violations.push(violation);
                        }
                    }

                    if loop_depth <= 0 {
                        in_loop = false;
                    }
                }
            }
            Ok(())
        },
    )?;

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

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, src_dir| {
            let path = &entry.absolute_path;
            if validator.should_skip_crate(src_dir)
                || path.to_str().is_some_and(|s| s.contains("/tests/"))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                if trimmed.starts_with("//") {
                    continue;
                }

                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                if in_test_module {
                    continue;
                }

                for (pattern, desc, sugg) in compiled_patterns {
                    if pattern.is_match(line) {
                        violations.push(make_violation(
                            path.to_path_buf(),
                            line_num + 1,
                            desc,
                            sugg,
                        ));
                    }
                }
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
