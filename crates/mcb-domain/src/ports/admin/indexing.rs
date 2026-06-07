//! Indexing operation tracking ports.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::value_objects::{CollectionId, OperationId};

/// Status of an indexing operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexingOperationStatus {
    /// Operation is starting
    Starting,
    /// Operation is in progress
    InProgress,
    /// Operation has completed successfully
    Completed,
    /// Operation has failed with an error message
    Failed(String),
}

/// Data about an ongoing indexing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingOperation {
    /// Unique identifier for the operation
    pub id: OperationId,
    /// Target collection for indexing
    pub collection: CollectionId,
    /// Current status of the operation
    pub status: IndexingOperationStatus,
    /// Total number of files to index
    pub total_files: usize,
    /// Number of files processed so far
    pub processed_files: usize,
    /// Current file being processed, if any
    pub current_file: Option<String>,
    /// Timestamp when the operation started
    pub started_at: i64,
}

/// Interface for tracking indexing operations
pub trait IndexingOperationsInterface: Send + Sync {
    /// Get all tracked indexing operations.
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation>;
    /// Start a new indexing operation.
    fn start_operation(&self, collection: &CollectionId, total_files: usize) -> OperationId;
    /// Update progress of an operation.
    fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed: usize,
    );
    /// Mark an operation as completed.
    fn complete_operation(&self, operation_id: &OperationId);
}
