//! Shared file-scanning helpers for validators.
//!
//! Provides both generic language-aware scan functions and backward-compatible
//! Rust-only wrappers. All filtering uses `InventoryEntry::detected_language`
//! from the single-pass file inventory — no extension-based hardcoding.

use std::path::Path;

use crate::filters::LanguageId;
use crate::run_context::{InventoryEntry, ValidationRunContext};
use crate::{Result, ValidationConfig};

/// True if a path points to a test file or tests directory.
#[must_use]
pub fn is_test_path(path: &str) -> bool {
    path.contains("_test.rs") || path.contains("/tests/")
}

// ---------------------------------------------------------------------------
// Generic, language-aware scan helpers
// ---------------------------------------------------------------------------

/// Iterate over files matching `language` in each crate's `src` directory.
///
/// When `language` is `None`, yields every file regardless of language.
///
/// # Errors
/// Returns an error if directory traversal fails or file access is denied.
pub fn for_each_crate_file<F>(
    config: &ValidationConfig,
    language: Option<LanguageId>,
    mut f: F,
) -> Result<()>
where
    F: FnMut(&InventoryEntry, &Path, &str) -> Result<()>,
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
            if !matches_language(entry, language) {
                continue;
            }
            f(entry, &src_dir, crate_name)?;
        }
    }

    Ok(())
}

/// Iterate over files matching `language` in configured scan directories.
///
/// When `language` is `None`, yields every file regardless of language.
///
/// # Errors
/// Returns an error if directory traversal fails.
pub fn for_each_scan_file<F>(
    config: &ValidationConfig,
    language: Option<LanguageId>,
    skip_validate_crate: bool,
    mut f: F,
) -> Result<()>
where
    F: FnMut(&InventoryEntry, &Path) -> Result<()>,
{
    let context = ValidationRunContext::active_or_build(config)?;
    let inventory = context.file_inventory();
    let file_config = crate::config::FileConfig::load(&config.workspace_root);

    for src_dir in config.get_scan_dirs()? {
        if skip_validate_crate
            && let Some(dir_name) = src_dir.file_name().and_then(|n| n.to_str())
            && file_config
                .general
                .skip_crates
                .iter()
                .any(|skip| dir_name.contains(skip))
        {
            continue;
        }

        for entry in inventory {
            if !entry.absolute_path.starts_with(&src_dir) {
                continue;
            }
            if !matches_language(entry, language) {
                continue;
            }
            f(entry, &src_dir)?;
        }
    }

    Ok(())
}

/// Iterate over files matching `language` under a root directory.
///
/// When `language` is `None`, yields every file regardless of language.
pub(crate) fn for_each_file_under_root<F>(
    config: &ValidationConfig,
    root: &Path,
    language: Option<LanguageId>,
    mut f: F,
) -> Result<()>
where
    F: FnMut(&InventoryEntry) -> Result<()>,
{
    if !root.exists() {
        return Ok(());
    }

    let context = ValidationRunContext::active_or_build(config)?;
    let inventory = context.file_inventory();
    let normalized_root = std::fs::canonicalize(root).map_err(|e| {
        crate::ValidationError::Config(format!(
            "Failed to canonicalize root {}: {e}",
            root.display()
        ))
    })?;

    for entry in inventory {
        if !entry.absolute_path.starts_with(&normalized_root) {
            continue;
        }
        if !matches_language(entry, language) {
            continue;
        }
        f(entry)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Rust-specific wrappers
// ---------------------------------------------------------------------------

/// Iterate over `.rs` files in each crate's `src` directory.
///
/// # Errors
/// Returns an error if directory traversal fails or file access is denied.
pub fn for_each_crate_rs_path<F>(config: &ValidationConfig, mut f: F) -> Result<()>
where
    F: FnMut(&Path, &Path, &str) -> Result<()>,
{
    for_each_crate_file(
        config,
        Some(LanguageId::Rust),
        |entry, src_dir, crate_name| f(&entry.absolute_path, src_dir, crate_name),
    )
}

/// Iterate over `.rs` files in configured scan directories.
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
    for_each_scan_file(
        config,
        Some(LanguageId::Rust),
        skip_validate_crate,
        |entry, src_dir| f(&entry.absolute_path, src_dir),
    )
}

/// Iterate over `.rs` files under a specific root directory.
pub fn for_each_rs_under_root<F>(config: &ValidationConfig, root: &Path, mut f: F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    for_each_file_under_root(config, root, Some(LanguageId::Rust), |entry| {
        f(&entry.absolute_path)
    })
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn matches_language(entry: &InventoryEntry, language: Option<LanguageId>) -> bool {
    match language {
        Some(lang) => entry.detected_language == Some(lang),
        None => true,
    }
}

/// Extracts a code block defined by balanced braces `{}` starting search from `start_line_idx`.
/// Returns the lines inclusive of the start and end lines, and the index of the last line.
pub fn extract_balanced_block<'a>(
    lines: &'a [&'a str],
    start_line_idx: usize,
) -> Option<(Vec<&'a str>, usize)> {
    let mut brace_balance = 0;
    let mut found_start = false;

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
                let block_lines = lines[start_line_idx..=current_idx].to_vec();
                return Some((block_lines, current_idx));
            }
        } else if offset > 20 {
            return None;
        }
    }

    None
}
