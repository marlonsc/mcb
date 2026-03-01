use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    /// Creates a guard that changes to `new_dir` and restores the original directory on drop.
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
