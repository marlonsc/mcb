//! RAII guards for test isolation — environment variables, current directory, file backup.
//!
//! Centralized in `mcb-domain` so every crate can reuse the same primitives.
#![allow(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// EnvVarGuard — temporarily set env vars, restore on drop
// ---------------------------------------------------------------------------

/// RAII guard that sets environment variables and restores them on drop.
///
/// # Example
/// ```rust,ignore
/// let _guard = EnvVarGuard::new(&[("MY_VAR", "value")]);
/// assert_eq!(std::env::var("MY_VAR").unwrap(), "value");
/// // MY_VAR removed when _guard is dropped
/// ```
pub struct EnvVarGuard {
    keys: Vec<String>,
}

impl EnvVarGuard {
    /// Set multiple env vars at once; all are removed on drop.
    #[must_use]
    pub fn new(vars: &[(&str, &str)]) -> Self {
        for (k, v) in vars {
            // SAFETY: Test-only helper; tests using env vars run serially
            // (not multi-threaded) so concurrent mutation is not a concern.
            unsafe {
                env::set_var(k, v);
            }
        }
        Self {
            keys: vars.iter().map(|(k, _)| (*k).to_owned()).collect(),
        }
    }

    /// Shorthand for a single key-value pair.
    #[must_use]
    pub fn set(key: &str, value: &str) -> Self {
        Self::new(&[(key, value)])
    }

    /// Remove env vars without a guard (immediate).
    pub fn remove(vars: &[&str]) {
        for key in vars {
            // SAFETY: Test-only helper; tests using env vars run serially.
            unsafe {
                env::remove_var(key);
            }
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        for key in &self.keys {
            // SAFETY: Test-only helper; tests using env vars run serially.
            unsafe {
                env::remove_var(key);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CurrentDirGuard — temporarily change cwd, restore on drop
// ---------------------------------------------------------------------------

/// RAII guard that changes the current directory and restores the original on drop.
pub struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    /// Changes to `new_dir` and restores the original directory on drop.
    ///
    /// # Errors
    ///
    /// Returns an error if `current_dir()` or `set_current_dir()` fails.
    pub fn new(new_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let original = env::current_dir()?;
        env::set_current_dir(new_dir)?;
        Ok(Self { original })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

// ---------------------------------------------------------------------------
// RestoreFileGuard — temporarily move a file, restore on drop
// ---------------------------------------------------------------------------

/// RAII guard that moves a file to a backup location and restores it on drop.
pub struct RestoreFileGuard {
    backup: PathBuf,
    target: PathBuf,
}

impl RestoreFileGuard {
    /// Moves `target` to `backup` and restores it on drop.
    ///
    /// # Errors
    ///
    /// Returns an error if the rename operation fails.
    pub fn move_out(target: &Path, backup: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        fs::rename(target, backup)?;
        Ok(Self {
            backup: backup.to_path_buf(),
            target: target.to_path_buf(),
        })
    }
}

impl Drop for RestoreFileGuard {
    fn drop(&mut self) {
        if self.backup.exists() {
            let _ = fs::rename(&self.backup, &self.target);
        }
    }
}
