use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::config::SyncBatch;
use crate::value_objects::{CollectionId, OperationId};
use async_trait::async_trait;
use std::path::Path;

/// Indexing Service Interface
///
/// Defines the contract for codebase indexing operations.
#[async_trait]
pub trait IndexingServiceInterface: Send + Sync {
    /// Index a codebase at the given path
    async fn index_codebase(
        &self,
        path: &Path,
        collection: &CollectionId,
    ) -> Result<IndexingResult>;

    /// Get the current indexing status
    fn get_status(&self) -> IndexingStatus;

    /// Clear all indexed data from a collection
    async fn clear_collection(&self, collection: &CollectionId) -> Result<()>;
}

/// Result of an indexing operation
#[derive(Debug, Clone)]
pub struct IndexingResult {
    /// Number of files processed
    pub files_processed: usize,
    /// Number of chunks created
    pub chunks_created: usize,
    /// Number of files skipped
    pub files_skipped: usize,
    /// Any errors encountered (non-fatal)
    pub errors: Vec<String>,
    /// Operation ID for async tracking (None for synchronous operations)
    pub operation_id: Option<OperationId>,
    /// Status string: "started", "completed", "failed"
    pub status: String,
}

/// Current indexing status
#[derive(Debug, Clone, Default)]
pub struct IndexingStatus {
    /// Whether indexing is currently in progress
    pub is_indexing: bool,
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
}

/// Domain Service: Advanced Batch Indexing Operations
///
/// Extended interface for batch indexing services that handle
/// incremental updates, rebuilds, and detailed statistics.
#[async_trait]
pub trait BatchIndexingServiceInterface: Send + Sync {
    /// Index a batch of code chunks
    async fn index_chunks(&self, chunks: &[CodeChunk]) -> Result<()>;

    /// Index files from a directory
    async fn index_directory(&self, path: &Path) -> Result<()>;

    /// Process a synchronization batch
    async fn process_sync_batch(&self, batch: &SyncBatch) -> Result<()>;

    /// Rebuild index for a collection
    async fn rebuild_index(&self, collection: &CollectionId) -> Result<()>;

    /// Get indexing statistics
    async fn get_indexing_stats(&self) -> Result<IndexingStats>;
}

/// Value Object: Indexing Statistics
#[derive(Debug, Clone)]
pub struct IndexingStats {
    /// Total number of chunks indexed
    pub total_chunks: u64,
    /// Total number of collections
    pub total_collections: u64,
    /// Last indexing operation timestamp
    pub last_indexed_at: Option<i64>,
    /// Average indexing throughput (chunks per second)
    pub avg_throughput: f64,
}
