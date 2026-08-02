//! Canonical path utilities — strict, no fallbacks.
//!
//! All functions in this module return `Result` on invalid input.
//! They never silently degrade (no `to_string_lossy`, no absolute-path fallbacks).

use std::path::{Path, PathBuf};

use crate::error::UtilsError;

/// Returns a workspace-relative path string with forward-slash separators.
///
/// # Errors
///
/// - `path` is not under `root` (`strip_prefix` fails)
/// - The relative path contains non-UTF-8 components
///
pub fn workspace_relative_path(path: &Path, root: &Path) -> Result<String, UtilsError> {
    let relative = strict_strip_prefix(path, root)?;
    path_to_utf8_string(&relative)
}

/// Strips `root` from `path`, returning the relative remainder.
///
/// # Errors
///
/// Returns an error if `path` is not under `root`.
pub fn strict_strip_prefix(path: &Path, root: &Path) -> Result<PathBuf, UtilsError> {
    path.strip_prefix(root)
        .map(std::path::Path::to_path_buf)
        .map_err(|_| {
            UtilsError::InvalidPath(format!(
                "path '{}' is not under root '{}'",
                path.display(),
                root.display()
            ))
        })
}

/// Converts a `Path` to a UTF-8 `String` with forward slashes.
///
/// # Errors
///
/// Returns an error if the path contains non-UTF-8 components.
pub fn path_to_utf8_string(path: &Path) -> Result<String, UtilsError> {
    let s = path.to_str().ok_or_else(|| {
        UtilsError::InvalidPath(format!(
            "path '{}' contains non-UTF-8 characters",
            path.display()
        ))
    })?;
    Ok(s.replace('\\', "/"))
}

/// Canonicalizes a path via the filesystem.
///
/// # Errors
///
/// Returns an error if `std::fs::canonicalize` fails (path does not exist, permission denied, etc.).
/// Unlike common fallback patterns, this **never** returns the original path on failure.
pub fn strict_canonicalize(path: &Path) -> Result<PathBuf, UtilsError> {
    std::fs::canonicalize(path).map_err(|e| {
        UtilsError::InvalidPath(format!("failed to canonicalize '{}': {e}", path.display()))
    })
}
