//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#repository-ports)
//!
//! Index Repository Port
//!
//! Provides persistent storage for indexing operations, enabling
//! database-backed progress tracking instead of in-memory only state.

use async_trait::async_trait;

use crate::error::Result;
use crate::ports::admin::IndexingOperation;
use crate::value_objects::{CollectionId, OperationId};

/// Repository for persisting indexing operation state in the database.
///
/// Unlike the in-memory `IndexingOperationsInterface`, this trait provides
/// durable storage of indexing progress that survives process restarts.
///
/// # Operations
///
/// - **Start**: Create a new indexing operation record
/// - **Status**: Query current progress and state
/// - **Update**: Track file-level progress
/// - **Complete/Fail**: Mark terminal states
/// - **Clear**: Remove all index data for a collection
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::IndexRepository;
/// use mcb_domain::value_objects::CollectionId;
/// use std::sync::Arc;
///
/// async fn track_indexing(repo: Arc<dyn IndexRepository>) -> mcb_domain::Result<()> {
///     let collection = CollectionId::from_name("my-project");
///     let op_id = repo.start_indexing(&collection, 100).await?;
///
///     // Update progress as files are processed
///     repo.update_progress(&op_id, Some("src/main.rs".to_string()), 1).await?;
///
///     // Check status
///     let op = repo.get_operation(&op_id).await?;
///     println!("Progress: {}/{}", op.unwrap().processed_files, 100);
///
///     // Mark complete
///     repo.complete_operation(&op_id).await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait IndexRepository: Send + Sync {
    /// Start a new indexing operation for a collection.
    ///
    /// Creates a persistent record of the operation with `Starting` status.
    ///
    /// # Arguments
    /// - `collection`: The collection being indexed
    /// - `total_files`: Total number of files to process
    ///
    /// # Returns
    /// A unique operation ID for tracking progress
    async fn start_indexing(
        &self,
        collection: &CollectionId,
        total_files: usize,
    ) -> Result<OperationId>;

    /// Get the current state of an indexing operation.
    ///
    /// # Returns
    /// `Some(operation)` if found, `None` if the operation ID is unknown
    async fn get_operation(&self, operation_id: &OperationId) -> Result<Option<IndexingOperation>>;

    /// Get all indexing operations (active and recent).
    ///
    /// # Returns
    /// All tracked operations, ordered by start time descending
    async fn list_operations(&self) -> Result<Vec<IndexingOperation>>;

    /// Get the active operation for a collection, if any.
    ///
    /// # Returns
    /// The currently running operation for the collection, or `None`
    async fn get_active_operation(
        &self,
        collection: &CollectionId,
    ) -> Result<Option<IndexingOperation>>;

    /// Update progress of an indexing operation.
    ///
    /// # Arguments
    /// - `operation_id`: The operation to update
    /// - `current_file`: The file currently being processed
    /// - `processed_files`: Total files processed so far
    async fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed_files: usize,
    ) -> Result<()>;

    /// Mark an operation as successfully completed.
    async fn complete_operation(&self, operation_id: &OperationId) -> Result<()>;

    /// Mark an operation as failed with an error message.
    async fn fail_operation(&self, operation_id: &OperationId, error: &str) -> Result<()>;

    /// Clear all index data for a collection.
    ///
    /// This removes:
    /// - All file hash records for the collection
    /// - The collection metadata record
    /// - Any active indexing operations for the collection
    ///
    /// # Returns
    /// Number of file hash records removed
    async fn clear_index(&self, collection: &CollectionId) -> Result<u64>;

    /// Get indexing statistics for a collection.
    ///
    /// # Returns
    /// - Total indexed files (active, non-tombstoned)
    /// - Last indexing timestamp
    async fn get_index_stats(&self, collection: &CollectionId) -> Result<IndexStats>;
}

/// Statistics about a collection's index state.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Number of actively indexed files
    pub indexed_files: u64,
    /// Timestamp of the most recent indexing operation
    pub last_indexed_at: Option<i64>,
    /// Whether an indexing operation is currently in progress
    pub is_indexing: bool,
}
