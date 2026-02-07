//! VCS repository registry for branch-scoped operations.
//!
//! Stores a mapping of repository_id -> path so tools like search_branch can
//! resolve a repository without requiring the path in every call.

use mcb_domain::entities::vcs::RepositoryId;
use mcb_domain::error::{Error, Result};
use mcb_infrastructure::config::{VCS_LOCK_FILENAME, VCS_REGISTRY_FILENAME, config_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct Registry {
    repositories: HashMap<RepositoryId, PathBuf>,
}

impl Registry {
    fn empty() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }
}

fn registry_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(VCS_REGISTRY_FILENAME))
}

fn lock_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(VCS_LOCK_FILENAME))
}

struct FileLockGuard {
    _file: std::fs::File,
}

use fs2::FileExt;

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

        file.lock_exclusive()
            .map_err(|e| Error::io(format!("Failed to acquire file lock: {e}")))?;

        Ok(Self { _file: file })
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
pub fn record_repository(repository_id: &RepositoryId, path: &Path) -> Result<()> {
    let _guard = FileLockGuard::acquire()?;
    let registry_path = registry_path()?;
    let mut registry = load_registry(&registry_path)?;
    registry
        .repositories
        .insert(repository_id.clone(), path.to_path_buf());
    save_registry(&registry_path, &registry)
}

/// Resolve a repository path by repository_id.
pub fn lookup_repository_path(repository_id: &RepositoryId) -> Result<PathBuf> {
    let _guard = FileLockGuard::acquire()?;
    let registry_path = registry_path()?;
    let registry = load_registry(&registry_path)?;
    registry
        .repositories
        .get(repository_id)
        .cloned()
        .ok_or_else(|| Error::repository_not_found(repository_id.to_string()))
}
