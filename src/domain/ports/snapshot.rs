//! Snapshot Provider Domain Port
//!
//! Defines the business contract for codebase snapshot operations. This abstraction
//! enables services to create, load, and compare snapshots without coupling to
//! specific storage or traversal implementations.

use crate::domain::error::Result;
use crate::domain::types::{CodebaseSnapshot, SnapshotChanges};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Domain Port for Codebase Snapshot Operations
///
/// This trait defines the contract for managing codebase snapshots used for
/// incremental indexing. Snapshots capture the state of files (paths, sizes,
/// modification times, hashes) to detect what has changed between indexing runs.
///
/// # Example
///
/// ```rust,ignore
/// use crate::domain::ports::snapshot::SnapshotProvider;
///
/// async fn get_files_to_reindex(
///     provider: &dyn SnapshotProvider,
///     root_path: &Path,
/// ) -> Result<Vec<String>> {
///     provider.get_changed_files(root_path).await
/// }
/// ```
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    /// Create a new snapshot for a codebase
    ///
    /// Traverses the codebase at `root_path`, computes file hashes, and creates
    /// a snapshot representing the current state. The snapshot is automatically
    /// saved to persistent storage.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root directory of the codebase to snapshot
    ///
    /// # Returns
    ///
    /// The created snapshot with all file metadata
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot>;

    /// Load an existing snapshot for a codebase
    ///
    /// Retrieves the most recent snapshot for the given codebase path.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root directory of the codebase
    ///
    /// # Returns
    ///
    /// The snapshot if one exists, None otherwise
    async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>>;

    /// Compare two snapshots to find changes
    ///
    /// Analyzes the differences between an old and new snapshot to determine
    /// which files were added, modified, removed, or unchanged.
    ///
    /// # Arguments
    ///
    /// * `old_snapshot` - Previous snapshot state
    /// * `new_snapshot` - Current snapshot state
    ///
    /// # Returns
    ///
    /// Detailed breakdown of changes between snapshots
    async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges>;

    /// Get files that need processing (added or modified since last snapshot)
    ///
    /// Convenience method that creates a new snapshot, compares with the previous
    /// one, and returns the list of files that need to be re-indexed.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root directory of the codebase
    ///
    /// # Returns
    ///
    /// List of file paths (relative to root_path) that have changed
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>>;
}

/// Shared snapshot provider for dependency injection
pub type SharedSnapshotProvider = Arc<dyn SnapshotProvider>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::FileSnapshot;
    use std::collections::HashMap;

    /// Mock snapshot provider for testing
    struct MockSnapshotProvider {
        stored_snapshot: Option<CodebaseSnapshot>,
    }

    impl MockSnapshotProvider {
        fn new() -> Self {
            Self {
                stored_snapshot: None,
            }
        }

        fn with_snapshot(snapshot: CodebaseSnapshot) -> Self {
            Self {
                stored_snapshot: Some(snapshot),
            }
        }
    }

    #[async_trait]
    impl SnapshotProvider for MockSnapshotProvider {
        async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot> {
            Ok(CodebaseSnapshot {
                root_path: root_path.to_string_lossy().to_string(),
                created_at: 1234567890,
                files: HashMap::new(),
                file_count: 0,
                total_size: 0,
            })
        }

        async fn load_snapshot(&self, _root_path: &Path) -> Result<Option<CodebaseSnapshot>> {
            Ok(self.stored_snapshot.clone())
        }

        async fn compare_snapshots(
            &self,
            old_snapshot: &CodebaseSnapshot,
            new_snapshot: &CodebaseSnapshot,
        ) -> Result<SnapshotChanges> {
            let mut added = Vec::new();
            let mut modified = Vec::new();
            let mut removed = Vec::new();
            let mut unchanged = Vec::new();

            for (path, new_file) in &new_snapshot.files {
                if let Some(old_file) = old_snapshot.files.get(path) {
                    if old_file.hash != new_file.hash {
                        modified.push(path.clone());
                    } else {
                        unchanged.push(path.clone());
                    }
                } else {
                    added.push(path.clone());
                }
            }

            for path in old_snapshot.files.keys() {
                if !new_snapshot.files.contains_key(path) {
                    removed.push(path.clone());
                }
            }

            Ok(SnapshotChanges {
                added,
                modified,
                removed,
                unchanged,
            })
        }

        async fn get_changed_files(&self, _root_path: &Path) -> Result<Vec<String>> {
            Ok(vec!["test.rs".to_string()])
        }
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let provider = MockSnapshotProvider::new();
        let result = provider.create_snapshot(Path::new("/test")).await;

        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert_eq!(snapshot.root_path, "/test");
    }

    #[tokio::test]
    async fn test_load_snapshot_when_exists() {
        let existing = CodebaseSnapshot {
            root_path: "/test".to_string(),
            created_at: 1234567890,
            files: HashMap::new(),
            file_count: 0,
            total_size: 0,
        };
        let provider = MockSnapshotProvider::with_snapshot(existing.clone());

        let result = provider.load_snapshot(Path::new("/test")).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_load_snapshot_when_not_exists() {
        let provider = MockSnapshotProvider::new();

        let result = provider.load_snapshot(Path::new("/test")).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_compare_snapshots() {
        let provider = MockSnapshotProvider::new();

        let mut old_files = HashMap::new();
        old_files.insert(
            "existing.rs".to_string(),
            FileSnapshot {
                path: "existing.rs".to_string(),
                size: 100,
                modified: 1000,
                hash: "abc123".to_string(),
                extension: "rs".to_string(),
            },
        );
        old_files.insert(
            "removed.rs".to_string(),
            FileSnapshot {
                path: "removed.rs".to_string(),
                size: 50,
                modified: 1000,
                hash: "def456".to_string(),
                extension: "rs".to_string(),
            },
        );

        let old_snapshot = CodebaseSnapshot {
            root_path: "/test".to_string(),
            created_at: 1000,
            files: old_files,
            file_count: 2,
            total_size: 150,
        };

        let mut new_files = HashMap::new();
        new_files.insert(
            "existing.rs".to_string(),
            FileSnapshot {
                path: "existing.rs".to_string(),
                size: 100,
                modified: 2000,
                hash: "xyz789".to_string(), // Modified
                extension: "rs".to_string(),
            },
        );
        new_files.insert(
            "added.rs".to_string(),
            FileSnapshot {
                path: "added.rs".to_string(),
                size: 200,
                modified: 2000,
                hash: "new123".to_string(),
                extension: "rs".to_string(),
            },
        );

        let new_snapshot = CodebaseSnapshot {
            root_path: "/test".to_string(),
            created_at: 2000,
            files: new_files,
            file_count: 2,
            total_size: 300,
        };

        let result = provider
            .compare_snapshots(&old_snapshot, &new_snapshot)
            .await;

        assert!(result.is_ok());
        let changes = result.unwrap();
        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.modified.len(), 1);
        assert_eq!(changes.removed.len(), 1);
        assert!(changes.added.contains(&"added.rs".to_string()));
        assert!(changes.modified.contains(&"existing.rs".to_string()));
        assert!(changes.removed.contains(&"removed.rs".to_string()));
    }

    #[tokio::test]
    async fn test_get_changed_files() {
        let provider = MockSnapshotProvider::new();
        let result = provider.get_changed_files(Path::new("/test")).await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(!files.is_empty());
    }
}
