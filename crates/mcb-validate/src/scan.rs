//! Shared file-scanning helpers for validators.

use std::path::Path;

use crate::run_context::ValidationRunContext;
use crate::{Result, ValidationConfig};

/// True if a path points to a test file or tests directory.
pub fn is_test_path(path: &str) -> bool {
    path.contains("_test.rs") || path.contains("/tests/")
}

/// Iterate over Rust source files in each crate's `src` directory.
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

            if !entry
                .absolute_path
                .extension()
                .is_some_and(|ext| ext == "rs")
            {
                continue;
            }

            f(&entry.absolute_path, &src_dir, crate_name)?;
        }
    }

    Ok(())
}

/// Iterate over Rust source files in configured scan directories.
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

            if !entry
                .absolute_path
                .extension()
                .is_some_and(|ext| ext == "rs")
            {
                continue;
            }

            f(&entry.absolute_path, &src_dir)?;
        }
    }

    Ok(())
}
