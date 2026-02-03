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
/// Provides semantic code search capabilities.
#[async_trait]
pub trait SearchServiceInterface: Send + Sync {
    /// Search for code similar to the query
    async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Search with optional filters for more refined results
    async fn search_with_filters(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
        filters: Option<&SearchFilters>,
    ) -> Result<Vec<SearchResult>>;
}

/// Filters for search queries
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Filter by file extension (e.g., "rs", "py")
    pub file_extensions: Option<Vec<String>>,
    /// Filter by programming language
    pub languages: Option<Vec<String>>,
    /// Minimum relevance score threshold (0.0 to 1.0)
    pub min_score: Option<f32>,
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
    /// Operation ID for async tracking (None for synchronous operations)
    pub operation_id: Option<String>,
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

// Re-export from domain layer (Clean Architecture)
pub use mcb_domain::ports::services::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};

// ============================================================================
// Memory Service Interface
// ============================================================================

pub use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
pub use mcb_domain::entities::memory::{
    MemoryFilter, MemorySearchResult, Observation, ObservationType, SessionSummary,
};
pub use mcb_domain::ports::repositories::agent_repository::AgentSessionQuery;

/// Memory Service Interface
///
/// Provides observation storage and retrieval with semantic search capabilities.
/// Supports session-based memory organization and content deduplication.
#[async_trait]
pub trait MemoryServiceInterface: Send + Sync {
    /// Store an observation with optional embedding for semantic search.
    ///
    /// Returns `(observation_id, deduplicated)`. If duplicate content is detected (same hash),
    /// returns the existing observation's ID and `deduplicated: true`.
    async fn store_observation(
        &self,
        content: String,
        observation_type: ObservationType,
        tags: Vec<String>,
        metadata: mcb_domain::entities::memory::ObservationMetadata,
    ) -> Result<(String, bool)>;

    /// Search memories using semantic similarity.
    ///
    /// Returns observations ranked by similarity to the query embedding.
    async fn search_memories(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>>;

    /// Get a session summary by session ID.
    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>>;

    /// Create or update a session summary.
    ///
    /// Summarizes the key topics, decisions, and next steps from a session.
    async fn create_session_summary(
        &self,
        session_id: String,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String>;

    /// Get an observation by ID.
    async fn get_observation(&self, id: &str) -> Result<Option<Observation>>;

    /// Generate embedding for content (for external use).
    async fn embed_content(&self, content: &str) -> Result<Embedding>;

    /// Get observations in timeline order around an anchor (for progressive disclosure).
    async fn get_timeline(
        &self,
        anchor_id: &str,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>>;

    /// Get multiple observations by IDs (for progressive disclosure step 3).
    async fn get_observations_by_ids(&self, ids: &[String]) -> Result<Vec<Observation>>;

    /// Token-efficient memory search - returns index only (no full content).
    ///
    /// This is Step 1 of the 3-layer workflow (search -> timeline -> details).
    /// Returns lightweight index entries with IDs, types, tags, scores, and brief previews.
    /// Use memory_get_observations with the returned IDs for full details.
    async fn memory_search(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<mcb_domain::entities::memory::MemorySearchIndex>>;
}

#[async_trait]
pub trait AgentSessionServiceInterface: Send + Sync {
    async fn create_session(&self, session: AgentSession) -> Result<String>;
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    async fn update_session(&self, session: AgentSession) -> Result<()>;
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()>;
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    async fn restore_checkpoint(&self, id: &str) -> Result<()>;
}
