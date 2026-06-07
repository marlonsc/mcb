//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Canonical path utilities — strict, no fallbacks.
//!
//! See [`UTILITIES_POLICY.md`](./UTILITIES_POLICY.md) for rules.
//!
//! All functions in this module return `Result` on invalid input.
//! They never silently degrade (no `to_string_lossy`, no absolute-path fallbacks).

use std::path::{Path, PathBuf};

use crate::error::Error;

/// Returns a workspace-relative path string with forward-slash separators.
///
/// # Errors
///
/// - `path` is not under `root` (`strip_prefix` fails)
/// - The relative path contains non-UTF-8 components
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use mcb_domain::utils::path::workspace_relative_path;
/// let root = Path::new("/home/user/project");
/// let file = Path::new("/home/user/project/src/main.rs");
/// assert_eq!(workspace_relative_path(file, root).unwrap(), "src/main.rs");
/// ```
pub fn workspace_relative_path(path: &Path, root: &Path) -> Result<String, Error> {
    let relative = strict_strip_prefix(path, root)?;
    path_to_utf8_string(&relative)
}

/// Strips `root` from `path`, returning the relative remainder.
///
/// # Errors
///
/// Returns an error if `path` is not under `root`.
pub fn strict_strip_prefix(path: &Path, root: &Path) -> Result<PathBuf, Error> {
    path.strip_prefix(root)
        .map(std::path::Path::to_path_buf)
        .map_err(|_| {
            Error::invalid_argument(format!(
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
pub fn path_to_utf8_string(path: &Path) -> Result<String, Error> {
    let s = path.to_str().ok_or_else(|| {
        Error::invalid_argument(format!(
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
pub fn strict_canonicalize(path: &Path) -> Result<PathBuf, Error> {
    std::fs::canonicalize(path).map_err(|e| {
        Error::invalid_argument(format!("failed to canonicalize '{}': {e}", path.display()))
    })
}
