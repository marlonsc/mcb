//! Cross-process synchronization using filesystem-based lockfiles
//!
//! Provides atomic coordination between multiple MCP instances using file locks.
//! Similar to proper-lockfile but implemented with fs2 for Rust.

use crate::core::error::{Error, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Lock metadata for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMetadata {
    pub pid: u32,
    pub hostname: String,
    pub codebase_path: String,
    pub acquired_at: String,
    pub instance_id: String,
}

/// Lockfile manager for cross-process coordination
pub struct CodebaseLockManager;

impl CodebaseLockManager {
    /// Lock directory for all lock files
    const LOCK_DIR: &'static str = ".context/locks";

    /// Lock file extension
    const LOCK_EXT: &'static str = ".lock";

    /// Metadata file extension
    const META_EXT: &'static str = ".lock.meta";

    /// Stale lock timeout (5 minutes)
    const STALE_TIMEOUT_SECS: u64 = 300;

    /// Create lock directory if it doesn't exist
    fn ensure_lock_dir() -> Result<PathBuf> {
        let lock_dir = dirs::home_dir()
            .ok_or_else(|| Error::internal("Cannot determine home directory"))?
            .join(Self::LOCK_DIR);

        fs::create_dir_all(&lock_dir)
            .map_err(|e| Error::internal(format!("Failed to create lock directory: {}", e)))?;

        Ok(lock_dir)
    }

    /// Generate lock filename from codebase path (MD5 hash)
    fn lock_filename(codebase_path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let path_str = codebase_path
            .canonicalize()
            .unwrap_or_else(|_| codebase_path.to_path_buf())
            .to_string_lossy()
            .to_string();

        let mut hasher = DefaultHasher::new();
        path_str.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Acquire lock for a codebase path
    /// Returns Some(cleanup_fn) if lock acquired, None if already locked
    pub async fn acquire_lock(
        codebase_path: &Path,
    ) -> Result<Option<Box<dyn FnOnce() -> Result<()> + Send>>> {
        let lock_dir = Self::ensure_lock_dir()?;
        let lock_name = Self::lock_filename(codebase_path);
        let lock_path = lock_dir.join(format!("{}{}", lock_name, Self::LOCK_EXT));
        let meta_path = lock_dir.join(format!("{}{}", lock_name, Self::META_EXT));

        // Try to open lock file (create if doesn't exist)
        let lock_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(e) => {
                return Err(Error::internal(format!(
                    "Failed to open lock file {}: {}",
                    lock_path.display(),
                    e
                )));
            }
        };

        // Try to acquire exclusive lock (non-blocking)
        match lock_file.try_lock_exclusive() {
            Ok(()) => {
                // Lock acquired successfully
                println!("[LOCK] Acquired lock for {}", codebase_path.display());

                // Write metadata
                Self::write_lock_metadata(&meta_path, codebase_path).await?;

                // Return cleanup function
                let cleanup = Box::new(move || {
                    // Release lock
                    let _ = lock_file.unlock();
                    // Remove metadata
                    let _ = fs::remove_file(&meta_path);
                    println!("[LOCK] Released lock for {}", codebase_path.display());
                    Ok(())
                });

                Ok(Some(cleanup))
            }
            Err(_) => {
                // Lock already held by another process
                println!("[LOCK] Lock already held for {}", codebase_path.display());
                Ok(None)
            }
        }
    }

    /// Write lock metadata for monitoring
    async fn write_lock_metadata(meta_path: &Path, codebase_path: &Path) -> Result<()> {
        let metadata = LockMetadata {
            pid: std::process::id(),
            hostname: hostname::get()
                .unwrap_or_else(|_| "unknown".into())
                .to_string_lossy()
                .to_string(),
            codebase_path: codebase_path.to_string_lossy().to_string(),
            acquired_at: chrono::Utc::now().to_rfc3339(),
            instance_id: uuid::Uuid::new_v4().to_string(),
        };

        let json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| Error::internal(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(meta_path, json)
            .map_err(|e| Error::internal(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// Clean up stale locks from dead processes
    pub async fn cleanup_stale_locks() -> Result<usize> {
        let lock_dir = Self::ensure_lock_dir()?;
        let mut cleaned = 0;

        // Find all metadata files
        for entry in fs::read_dir(&lock_dir)
            .map_err(|e| Error::internal(format!("Failed to read lock directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| Error::internal(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            // Only process metadata files
            if let Some(ext) = path.extension() {
                if ext != Self::META_EXT.trim_start_matches('.') {
                    continue;
                }
            } else {
                continue;
            }

            // Check if metadata file is stale
            if Self::is_stale_lock(&path).await? {
                println!("[LOCK] Removing stale lock: {}", path.display());

                // Remove metadata file
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!(
                        "[LOCK] Failed to remove stale metadata {}: {}",
                        path.display(),
                        e
                    );
                    continue;
                }

                // Try to remove corresponding lock file
                let lock_path = path.with_extension(Self::LOCK_EXT.trim_start_matches('.'));
                if lock_path.exists() {
                    if let Err(e) = fs::remove_file(&lock_path) {
                        eprintln!(
                            "[LOCK] Failed to remove stale lock {}: {}",
                            lock_path.display(),
                            e
                        );
                    }
                }

                cleaned += 1;
            }
        }

        Ok(cleaned)
    }

    /// Check if a lock is stale (process no longer exists or file is too old)
    async fn is_stale_lock(meta_path: &Path) -> Result<bool> {
        // Check file modification time
        let metadata = fs::metadata(meta_path)
            .map_err(|e| Error::internal(format!("Failed to read metadata file: {}", e)))?;

        let modified = metadata
            .modified()
            .map_err(|e| Error::internal(format!("Failed to get file modification time: {}", e)))?;

        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(0));

        if age > Duration::from_secs(Self::STALE_TIMEOUT_SECS) {
            return Ok(true);
        }

        // Check if process still exists
        let content = fs::read_to_string(meta_path)
            .map_err(|e| Error::internal(format!("Failed to read metadata: {}", e)))?;

        let metadata: LockMetadata = serde_json::from_str(&content)
            .map_err(|e| Error::internal(format!("Failed to parse metadata: {}", e)))?;

        // Check if process exists (Unix-only for now)
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            use std::process::Command;

            // Use kill with signal 0 to check if process exists
            match Command::new("kill")
                .arg("-0")
                .arg(metadata.pid.to_string())
                .output()
            {
                Ok(output) => {
                    // If kill returns 0, process exists
                    Ok(output.status.code() != Some(0))
                }
                Err(_) => {
                    // If we can't check, assume it's not stale to be safe
                    Ok(false)
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, just check file age
            Ok(false)
        }
    }

    /// Get all active locks for monitoring
    pub async fn get_active_locks() -> Result<Vec<LockMetadata>> {
        let lock_dir = Self::ensure_lock_dir()?;
        let mut locks = Vec::new();

        for entry in fs::read_dir(&lock_dir)
            .map_err(|e| Error::internal(format!("Failed to read lock directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| Error::internal(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == Self::META_EXT.trim_start_matches('.') {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            if let Ok(metadata) = serde_json::from_str::<LockMetadata>(&content) {
                                locks.push(metadata);
                            }
                        }
                        Err(e) => {
                            eprintln!("[LOCK] Failed to read metadata {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(locks)
    }
}
