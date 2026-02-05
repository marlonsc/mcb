//! VCS repository registry for branch-scoped operations.
//!
//! Stores a mapping of repository_id -> path so tools like search_branch can
//! resolve a repository without requiring the path in every call.

use mcb_domain::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// Registry file name (stored under ~/.config/mcb)
const REGISTRY_FILENAME: &str = "vcs_repository_registry.json";
/// Lock file name to serialize registry writes
const LOCK_FILENAME: &str = "vcs_repository_registry.lock";

#[derive(Debug, Serialize, Deserialize)]
struct Registry {
    repositories: HashMap<String, String>,
}

impl Registry {
    fn empty() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }
}

fn registry_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::io("Unable to determine config directory"))?;
    Ok(config_dir.join("mcb").join(REGISTRY_FILENAME))
}

fn lock_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::io("Unable to determine config directory"))?;
    Ok(config_dir.join("mcb").join(LOCK_FILENAME))
}

struct FileLockGuard {
    _file: std::fs::File,
}

impl FileLockGuard {
    fn acquire() -> Result<Self> {
        let lock_path = lock_path()?;
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("Failed to create config directory: {e}")))?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| Error::io(format!("Failed to open lock file: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            // SAFETY: fd is a valid file descriptor obtained from AsRawFd trait.
            // libc::LOCK_EX is a valid flock operation constant.
            // flock only modifies kernel state, not memory safety.
            let result = unsafe { libc::flock(fd, libc::LOCK_EX) };
            if result != 0 {
                return Err(Error::io("Failed to acquire file lock"));
            }
        }

        Ok(Self { _file: file })
    }
}

#[cfg(unix)]
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        use std::os::unix::io::AsRawFd;
        let fd = self._file.as_raw_fd();
        // SAFETY: fd is a valid file descriptor obtained from AsRawFd trait.
        // libc::LOCK_UN is a valid flock operation constant for unlocking.
        // This call reverts the lock acquired in new().
        unsafe { libc::flock(fd, libc::LOCK_UN) };
    }
}

fn load_registry(path: &Path) -> Result<Registry> {
    if !path.exists() {
        return Ok(Registry::empty());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::io(format!("Failed to read registry: {e}")))?;
    if content.trim().is_empty() {
        return Ok(Registry::empty());
    }
    serde_json::from_str(&content).map_err(|e| Error::io(format!("Failed to parse registry: {e}")))
}

fn save_registry(path: &Path, registry: &Registry) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Failed to create config directory: {e}")))?;
    }
    let content = serde_json::to_string_pretty(registry)
        .map_err(|e| Error::io(format!("Failed to serialize registry: {e}")))?;
    std::fs::write(path, content)
        .map_err(|e| Error::io(format!("Failed to write registry: {e}")))?;
    Ok(())
}

/// Record the repository path for a given repository_id.
pub fn record_repository(repository_id: &str, path: &Path) -> Result<()> {
    let _guard = FileLockGuard::acquire()?;
    let registry_path = registry_path()?;
    let mut registry = load_registry(&registry_path)?;
    registry
        .repositories
        .insert(repository_id.to_string(), path.display().to_string());
    save_registry(&registry_path, &registry)
}

/// Resolve a repository path by repository_id.
pub fn lookup_repository_path(repository_id: &str) -> Result<PathBuf> {
    let _guard = FileLockGuard::acquire()?;
    let registry_path = registry_path()?;
    let registry = load_registry(&registry_path)?;
    registry
        .repositories
        .get(repository_id)
        .map(PathBuf::from)
        .ok_or_else(|| {
            Error::io(format!(
                "Repository id not found in registry: {repository_id}"
            ))
        })
}
