//! Snapshot, sync provider, and sync coordination ports.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::entities::codebase::{CodebaseSnapshot, SnapshotChanges};
use crate::error::Result;
use crate::value_objects::config::SyncBatch;

/// Sync Provider Interface
#[async_trait]
pub trait SyncProvider: Send + Sync {
    /// Check if a sync operation should be debounced for the given path.
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;
    /// Update the timestamp of the last successful sync.
    async fn update_last_sync(&self, codebase_path: &Path);
    /// Attempt to acquire a slot for a sync batch.
    async fn acquire_sync_slot(&self, codebase_path: &Path) -> Result<Option<SyncBatch>>;
    /// Release a previously acquired sync slot.
    async fn release_sync_slot(&self, codebase_path: &Path, batch: SyncBatch) -> Result<()>;
    /// Get list of files that have changed since last sync.
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;
    /// Desired interval between syncs.
    fn sync_interval(&self) -> Duration;
    /// Desired debounce duration.
    fn debounce_interval(&self) -> Duration;
}

/// Snapshot Provider Interface
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    /// Create a new snapshot of the filesystem at `root_path`.
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot>;
    /// Load a previously saved snapshot for `root_path`.
    async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>>;
    /// Compare two snapshots and find the differences.
    async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges>;
    /// Efficiently get files changed on disk since last snapshot.
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>>;
}

/// Configuration for sync operations
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Minimum time between consecutive sync attempts
    pub debounce_duration: Duration,
    /// Whether to force a sync even if debouncing would normally skip it
    pub force: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_secs(60),
            force: false,
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether the sync operation actually ran
    pub performed: bool,
    /// Number of files identified as changed
    pub files_changed: usize,
    /// List of paths for the changed files
    pub changed_files: Vec<String>,
}

impl SyncResult {
    /// Create a result representing a skipped operation (e.g., due to debouncing).
    #[must_use]
    pub fn skipped() -> Self {
        Self {
            performed: false,
            files_changed: 0,
            changed_files: Vec::new(),
        }
    }

    /// Create a result for a completed operation with the list of changes.
    #[must_use]
    pub fn completed(changed_files: Vec<String>) -> Self {
        let files_changed = changed_files.len();
        Self {
            performed: true,
            files_changed,
            changed_files,
        }
    }
}

/// Domain Port for File Synchronization Coordination
#[async_trait]
pub trait SyncCoordinator: Send + Sync {
    /// Check if a sync should be debounced for the given path.
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;
    /// Perform the synchronization operation.
    async fn sync(&self, codebase_path: &Path, options: SyncOptions) -> Result<SyncResult>;
    /// Get list of changed files according to the coordinator's state.
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;
    /// Explicitly mark a path as successfully synced.
    async fn mark_synced(&self, codebase_path: &Path) -> Result<()>;
    /// Total number of files currently tracked by the coordinator.
    fn tracked_file_count(&self) -> usize;
}

/// Shared sync coordinator for dependency injection
pub type SharedSyncCoordinator = Arc<dyn SyncCoordinator>;
