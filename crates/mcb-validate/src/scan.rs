//! Shared file-scanning helpers for validators.

use std::path::Path;

use crate::run_context::ValidationRunContext;
use crate::{Result, ValidationConfig};

/// True if a path points to a test file or tests directory.
#[must_use]
pub fn is_test_path(path: &str) -> bool {
    path.contains("_test.rs") || path.contains("/tests/")
}

/// Iterate over Rust source files in each crate's `src` directory.
///
/// # Errors
/// Returns an error if directory traversal fails or file access is denied.
pub fn for_each_crate_rs_path<F>(config: &ValidationConfig, mut f: F) -> Result<()>
where
    F: FnMut(&Path, &Path, &str) -> Result<()>,
{
    let context = ValidationRunContext::active_or_build(config)?;
    let inventory = context.file_inventory();

    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        let crate_name = crate_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for entry in inventory {
            if !entry.absolute_path.starts_with(&src_dir) {
                continue;
            }

            if entry
                .absolute_path
                .extension()
                .is_none_or(|ext| ext != "rs")
            {
                continue;
            }

            f(&entry.absolute_path, &src_dir, crate_name)?;
        }
    }

    Ok(())
}

/// Iterate over Rust source files in configured scan directories.
///
/// # Errors
/// Returns an error if directory traversal fails.
pub fn for_each_scan_rs_path<F>(
    config: &ValidationConfig,
    skip_validate_crate: bool,
    mut f: F,
) -> Result<()>
where
    F: FnMut(&Path, &Path) -> Result<()>,
{
    let context = ValidationRunContext::active_or_build(config)?;
    let inventory = context.file_inventory();

    // Load file configuration to get skip_crates
    let file_config = crate::config::FileConfig::load(&config.workspace_root);

    for src_dir in config.get_scan_dirs()? {
        if skip_validate_crate {
            // Skip any crates in the skip_crates list (typically includes the validate crate itself)
            if let Some(dir_name) = src_dir.file_name().and_then(|n| n.to_str())
                && file_config
                    .general
                    .skip_crates
                    .iter()
                    .any(|skip| dir_name.contains(skip))
            {
                continue;
            }
        }

        for entry in inventory {
            if !entry.absolute_path.starts_with(&src_dir) {
                continue;
            }

            if entry
                .absolute_path
                .extension()
                .is_none_or(|ext| ext != "rs")
            {
                continue;
            }

            f(&entry.absolute_path, &src_dir)?;
        }
    }

    Ok(())
}

pub(crate) fn for_each_rs_under_root<F>(
    config: &ValidationConfig,
    root: &Path,
    mut f: F,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    if !root.exists() {
        return Ok(());
    }

    let context = ValidationRunContext::active_or_build(config)?;
    let inventory = context.file_inventory();
    let normalized_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

    for entry in inventory {
        if !entry.absolute_path.starts_with(&normalized_root) {
            continue;
        }

        if entry
            .absolute_path
            .extension()
            .is_none_or(|ext| ext != "rs")
        {
            continue;
        }

        f(&entry.absolute_path)?;
    }

    Ok(())
}

/// Extracts a code block defined by balanced braces `{}` starting search from `start_line_idx`.
/// Returns the lines inclusive of the start and end lines, and the index of the last line.
///
/// This is a generic helper to reduce complexity in validators that need to analyze
/// function bodies, trait definitions, or other block structures.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn extract_balanced_block<'a>(
    lines: &'a [&'a str],
    start_line_idx: usize,
) -> Option<(Vec<&'a str>, usize)> {
    let mut brace_balance = 0;
    let mut found_start = false;

    // Safety check: don't scan if start is out of bounds
    if start_line_idx >= lines.len() {
        return None;
    }

    for (offset, line) in lines[start_line_idx..].iter().enumerate() {
        let current_idx = start_line_idx + offset;

        let open_count = line.chars().filter(|c| *c == '{').count() as i32;
        let close_count = line.chars().filter(|c| *c == '}').count() as i32;

        if open_count > 0 {
            found_start = true;
        }

        if found_start {
            brace_balance += open_count;
            brace_balance -= close_count;

            if brace_balance <= 0 {
                // Ensure we return a vector of references to strings
                let block_lines = lines[start_line_idx..=current_idx].to_vec();
                return Some((block_lines, current_idx));
            }
        } else if offset > 20 {
            // Heuristic: if we don't find an opening brace within 20 lines, it's likely not a block definition
            // (e.g. huge function signature or comments)
            return None;
        }
    }

    None
}
