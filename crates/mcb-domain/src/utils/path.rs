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

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn workspace_relative_happy_path() -> TestResult {
        let root = Path::new("/home/user/project");
        let file = Path::new("/home/user/project/src/main.rs");
        assert_eq!(workspace_relative_path(file, root)?, "src/main.rs");
        Ok(())
    }

    #[test]
    fn workspace_relative_nested() -> TestResult {
        let root = Path::new("/a/b");
        let file = Path::new("/a/b/c/d/e.rs");
        assert_eq!(workspace_relative_path(file, root)?, "c/d/e.rs");
        Ok(())
    }

    #[test]
    fn workspace_relative_outside_root_returns_error() {
        let root = Path::new("/home/user/project");
        let file = Path::new("/other/place/file.rs");
        let result = workspace_relative_path(file, root);
        assert!(result.is_err(), "expected error for path outside root");
        if let Err(err) = result {
            assert!(
                err.to_string().contains("is not under root"),
                "unexpected error: {err}"
            );
        }
    }

    #[test]
    fn strict_strip_prefix_same_path() -> TestResult {
        let root = Path::new("/a/b");
        let path = Path::new("/a/b");
        let result = strict_strip_prefix(path, root)?;
        assert_eq!(result, PathBuf::from(""));
        Ok(())
    }

    #[test]
    fn strict_strip_prefix_errors_outside_root() {
        let root = Path::new("/a/b");
        let path = Path::new("/a/c/d");
        assert!(strict_strip_prefix(path, root).is_err());
    }

    #[test]
    fn path_to_utf8_string_forward_slashes() -> TestResult {
        // On Unix the backslash is a valid filename char, but we still replace it
        let p = Path::new("src/main.rs");
        assert_eq!(path_to_utf8_string(p)?, "src/main.rs");
        Ok(())
    }

    #[test]
    fn strict_canonicalize_nonexistent_path_returns_error() {
        let bad = Path::new("/this/path/definitely/does/not/exist/xyz123");
        let result = strict_canonicalize(bad);
        assert!(result.is_err(), "expected error for nonexistent path");
        if let Err(err) = result {
            assert!(
                err.to_string().contains("failed to canonicalize"),
                "unexpected error: {err}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn strict_canonicalize_real_path_succeeds() -> TestResult {
        // /tmp always exists on Linux
        let result = strict_canonicalize(Path::new("/tmp"))?;
        // Canonicalized /tmp might resolve symlinks, but should succeed
        assert!(result.exists());
        Ok(())
    }
}
