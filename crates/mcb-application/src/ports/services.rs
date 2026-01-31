//! Application Service Port Interfaces
//!
//! Defines the port interfaces for application layer services.
//! These traits are the contracts that application services must implement,
//! following Clean Architecture principles.

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::value_objects::config::SyncBatch;
use mcb_domain::value_objects::{Embedding, SearchResult};
use std::path::Path;

// ============================================================================
// Context Service Interface
// ============================================================================

/// Code Intelligence Service Interface
///
/// Defines the contract for semantic code understanding operations.
#[async_trait]
pub trait ContextServiceInterface: Send + Sync {
    /// Initialize the service for a collection
    async fn initialize(&self, collection: &str) -> Result<()>;

    /// Store code chunks in the repository
    async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;

    /// Search for code similar to the query
    async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Get embedding for text
    async fn embed_text(&self, text: &str) -> Result<Embedding>;

    /// Clear/delete a collection
    async fn clear_collection(&self, collection: &str) -> Result<()>;

    /// Get combined stats for the service
    async fn get_stats(&self) -> Result<(i64, i64)>;

    /// Get embedding dimensions
    fn embedding_dimensions(&self) -> usize;
}

// ============================================================================
// Search Service Interface
// ============================================================================

/// Search Service Interface
///
/// Simplified search interface for code queries.
#[async_trait]
pub trait SearchServiceInterface: Send + Sync {
    /// Search for code similar to the query
    async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}

// ============================================================================
// Indexing Service Interface
// ============================================================================

/// Indexing Service Interface
///
/// Defines the contract for codebase indexing operations.
#[async_trait]
pub trait IndexingServiceInterface: Send + Sync {
    /// Index a codebase at the given path
    async fn index_codebase(&self, path: &Path, collection: &str) -> Result<IndexingResult>;

    /// Get the current indexing status
    fn get_status(&self) -> IndexingStatus;

    /// Clear all indexed data from a collection
    async fn clear_collection(&self, collection: &str) -> Result<()>;
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

// ============================================================================
// Chunking Orchestrator Interface
// ============================================================================

/// Chunking Orchestrator Interface
///
/// Coordinates batch code chunking operations.
#[async_trait]
pub trait ChunkingOrchestratorInterface: Send + Sync {
    /// Process multiple files and return chunks
    async fn process_files(&self, files: Vec<(String, String)>) -> Result<Vec<CodeChunk>>;

    /// Process a single file with content
    async fn process_file(&self, path: &Path, content: &str) -> Result<Vec<CodeChunk>>;

    /// Read and chunk a file from disk
    async fn chunk_file(&self, path: &Path) -> Result<Vec<CodeChunk>>;
}

// ============================================================================
// Batch Indexing Service Interface
// ============================================================================

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
    async fn rebuild_index(&self, collection: &str) -> Result<()>;

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

// ============================================================================
// Validation Service Interface
// ============================================================================

/// Report containing validation results
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationReport {
    /// Total number of violations found
    pub total_violations: usize,
    /// Number of error-level violations
    pub errors: usize,
    /// Number of warning-level violations
    pub warnings: usize,
    /// Number of info-level violations
    pub infos: usize,
    /// All violations found
    pub violations: Vec<ViolationEntry>,
    /// Whether validation passed (no error-level violations)
    pub passed: bool,
}

/// A single violation entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct ViolationEntry {
    /// Unique violation ID (e.g., "CA001", "SOLID002")
    pub id: String,
    /// Category (e.g., "clean_architecture", "solid", "quality")
    pub category: String,
    /// Severity level: "ERROR", "WARNING", or "INFO"
    pub severity: String,
    /// File path where violation was found (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Human-readable description of the violation
    pub message: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

/// Architecture Validation Service Interface
///
/// Defines the contract for running architecture validation on a codebase.
/// Implementations should delegate to mcb-validate for actual validation logic.
#[async_trait]
pub trait ValidationServiceInterface: Send + Sync {
    /// Validate a workspace against architecture rules
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    /// * `validators` - Optional list of specific validators to run
    /// * `severity_filter` - Optional minimum severity filter ("error", "warning", or "info")
    ///
    /// # Returns
    ///
    /// A `ValidationReport` containing all violations found.
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport>;

    /// List available validator names
    ///
    /// Returns a list of all validator names that can be passed to `validate()`.
    async fn list_validators(&self) -> Result<Vec<String>>;
}
