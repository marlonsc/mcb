//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
//! Shared file-scanning helpers for validators.
//!
//! Provides generic language-aware scan functions. All filtering uses
//! `InventoryEntry::detected_language` from the single-pass file inventory.

use std::path::Path;

use crate::ValidationConfigExt;
use crate::constants::common::{MAX_BLOCK_SEARCH_OFFSET, TEST_DIR_FRAGMENT, TEST_FILE_SUFFIX};
use crate::filters::LanguageId;
use crate::run_context::{InventoryEntry, ValidationRunContext};
use crate::{Result, ValidationConfig};

/// True if a path points to a test file or tests directory.
#[must_use]
pub fn is_test_path(path: &str) -> bool {
    path.contains(TEST_FILE_SUFFIX) || path.contains(TEST_DIR_FRAGMENT)
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
    let normalized_root = mcb_domain::utils::path::strict_canonicalize(root)
        .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

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
// Shared helpers
// ---------------------------------------------------------------------------

fn matches_language(entry: &InventoryEntry, language: Option<LanguageId>) -> bool {
    match language {
        Some(lang) => entry.detected_language == Some(lang),
        None => true,
    }
}

/// Extracts a code block defined by balanced braces `{}` starting search from `start_line_idx`.
#[must_use]
pub fn extract_balanced_block<'a>(
    lines: &'a [&'a str],
    start_line_idx: usize,
) -> Option<(Vec<&'a str>, usize)> {
    let count = mcb_domain::utils::analysis::count_balanced_block_lines(
        &lines[start_line_idx..],
        MAX_BLOCK_SEARCH_OFFSET,
    )?;
    let end_idx = start_line_idx + count - 1;
    Some((lines[start_line_idx..=end_idx].to_vec(), end_idx))
}
