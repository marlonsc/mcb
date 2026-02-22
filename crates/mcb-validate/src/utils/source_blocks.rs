//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use regex::Regex;

use crate::pattern_registry::required_pattern;
use crate::{Result, ValidationConfig};

use super::for_each_rust_file;

/// Visit lines inside a brace-balanced block starting at `start_line`.
pub fn within_block<F>(lines: &[&str], start_line: usize, mut visitor: F)
where
    F: FnMut(&str, usize) -> bool,
{
    let mut start_offset = 0;
    let slice = &lines[start_line..];
    while start_offset < slice.len() && !slice[start_offset].contains('{') {
        start_offset += 1;
    }

    if start_offset < slice.len()
        && let Some(count) =
            mcb_domain::utils::analysis::count_balanced_block_lines(slice, usize::MAX)
    {
        for (offset, line) in slice[start_offset..count].iter().enumerate() {
            if !visitor(line, start_offset + offset) {
                break;
            }
        }
    }
}

/// Count lines in a brace-balanced block.
#[must_use]
pub fn count_block_lines(lines: &[&str], start_line: usize) -> usize {
    let mut start_offset = 0;
    let slice = &lines[start_line..];
    while start_offset < slice.len() && !slice[start_offset].contains('{') {
        start_offset += 1;
    }
    if let Some(count) = mcb_domain::utils::analysis::count_balanced_block_lines(slice, usize::MAX)
        && count > start_offset
    {
        return count - start_offset;
    }
    0
}

/// Count regex matches inside a brace-balanced block.
#[must_use]
pub fn count_matches_in_block(lines: &[&str], start_line: usize, pattern: &Regex) -> usize {
    let mut count = 0;
    within_block(lines, start_line, |line, _| {
        if pattern.is_match(line) {
            count += 1;
        }
        true
    });
    count
}

/// Count match arms in a `match` block.
///
/// # Errors
/// Returns an error if the required arm-pattern regex is unavailable.
pub fn count_match_arms(lines: &[&str], start_line: usize) -> Result<usize> {
    let arrow_pattern = required_pattern("SOLID003.match_arrow")?;

    let mut count = 0;
    let mut brace_depth = 0;

    within_block(lines, start_line, |line, _| {
        brace_depth += line.chars().filter(|c| *c == '{').count();
        brace_depth -= line.chars().filter(|c| *c == '}').count();

        if arrow_pattern.is_match(line) && brace_depth >= 1 {
            count += 1;
        }
        true
    });
    Ok(count)
}

/// Scan declarations in Rust files and emit violations when member count exceeds threshold.
///
/// # Errors
/// Returns an error if file traversal fails.
pub fn scan_decl_blocks<F, V>(
    config: &ValidationConfig,
    scan_config: &DeclScanConfig<'_>,
    make_violation: F,
) -> Result<Vec<V>>
where
    F: Fn(PathBuf, usize, &str, usize, usize) -> V,
{
    let mut violations = Vec::new();

    for_each_rust_file(config, |path, lines| {
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(cap) = scan_config.decl_pattern.captures(line) {
                let name = cap.get(1).map_or("", |m| m.as_str());
                let method_count =
                    (scan_config.count_fn)(&lines, line_num, scan_config.member_fn_pattern);

                if method_count > scan_config.max_allowed {
                    violations.push(make_violation(
                        path.clone(),
                        line_num + 1,
                        name,
                        method_count,
                        scan_config.max_allowed,
                    ));
                }
            }
        }
        Ok(())
    })?;

    Ok(violations)
}

/// Configuration for declaration-block scans.
pub struct DeclScanConfig<'a> {
    /// Regex that captures declaration name in group 1.
    pub decl_pattern: &'a Regex,
    /// Regex used to count declaration members.
    pub member_fn_pattern: &'a Regex,
    /// Counting strategy for block members.
    pub count_fn: fn(&[&str], usize, &Regex) -> usize,
    /// Allowed maximum members before creating a violation.
    pub max_allowed: usize,
}
