//! Indexing operation repository ports.

use async_trait::async_trait;

use crate::error::Result;
use crate::ports::admin::indexing::IndexingOperation;
use crate::value_objects::{CollectionId, OperationId};

/// Statistics about a collection's index state.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Number of actively indexed files.
    pub indexed_files: u64,
    /// Timestamp of the most recent indexing operation.
    pub last_indexed_at: Option<i64>,
    /// Whether an indexing operation is currently in progress.
    pub is_indexing: bool,
}

/// Repository for persisting indexing operation state.
#[async_trait]
pub trait IndexRepository: Send + Sync {
    /// Start a new indexing operation for a collection.
    async fn start_indexing(
        &self,
        collection: &CollectionId,
        total_files: usize,
    ) -> Result<OperationId>;
    /// Get the current state of an indexing operation.
    async fn get_operation(&self, operation_id: &OperationId) -> Result<Option<IndexingOperation>>;
    /// Get all indexing operations (active and recent).
    async fn list_operations(&self) -> Result<Vec<IndexingOperation>>;
    /// Get the active operation for a collection, if any.
    async fn get_active_operation(
        &self,
        collection: &CollectionId,
    ) -> Result<Option<IndexingOperation>>;
    /// Update progress of an indexing operation.
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
    async fn clear_index(&self, collection: &CollectionId) -> Result<u64>;
    /// Get indexing statistics for a collection.
    async fn get_index_stats(&self, collection: &CollectionId) -> Result<IndexStats>;
}
