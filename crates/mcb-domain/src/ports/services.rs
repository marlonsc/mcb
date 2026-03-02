//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#service-ports)
//!
//! Domain service port interfaces for core business operations.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entities::CodeChunk;
use crate::entities::agent::{AgentSession, AgentSessionStatus, Checkpoint, Delegation, ToolCall};
use crate::entities::memory::{
    ErrorPattern, MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation,
    ObservationMetadata, ObservationType, OriginContext, SessionSummary,
};
use crate::entities::project::ProjectType;
use crate::error::Result;
use crate::ports::AgentSessionQuery;
use crate::value_objects::Language;
use crate::value_objects::browse::{FileNode, HighlightedCode};
use crate::value_objects::config::SyncBatch;
use crate::value_objects::{
    CollectionId, Embedding, ObservationId, OperationId, SearchResult, SessionId,
};

// ============================================================================
// Agent Service Port
// ============================================================================

/// Manages agent session lifecycle.
#[async_trait]
pub trait AgentSessionManager: Send + Sync {
    /// Performs the create session operation.
    async fn create_session(&self, session: AgentSession) -> Result<String>;
    /// Performs the get session operation.
    async fn get_session(&self, id: &str) -> Result<Option<AgentSession>>;
    /// Performs the update session operation.
    async fn update_session(&self, session: AgentSession) -> Result<()>;
    /// Performs the list sessions operation.
    async fn list_sessions(&self, query: AgentSessionQuery) -> Result<Vec<AgentSession>>;
    /// Performs the list sessions by project operation.
    async fn list_sessions_by_project(&self, project_id: &str) -> Result<Vec<AgentSession>>;
    /// Performs the list sessions by worktree operation.
    async fn list_sessions_by_worktree(&self, worktree_id: &str) -> Result<Vec<AgentSession>>;
    /// Performs the end session operation.
    async fn end_session(
        &self,
        id: &str,
        status: AgentSessionStatus,
        result_summary: Option<String>,
    ) -> Result<()>;
}

/// Tracks delegations and tool calls.
#[async_trait]
pub trait DelegationTracker: Send + Sync {
    /// Performs the store delegation operation.
    async fn store_delegation(&self, delegation: Delegation) -> Result<String>;
    /// Performs the store tool call operation.
    async fn store_tool_call(&self, tool_call: ToolCall) -> Result<String>;
}

/// Manages checkpoints and restoration.
#[async_trait]
pub trait CheckpointManager: Send + Sync {
    /// Performs the store checkpoint operation.
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<String>;
    /// Performs the get checkpoint operation.
    async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>>;
    /// Performs the restore checkpoint operation.
    async fn restore_checkpoint(&self, id: &str) -> Result<()>;
}

/// Aggregate trait for agent session service.
pub trait AgentSessionServiceInterface:
    AgentSessionManager + DelegationTracker + CheckpointManager + Send + Sync
{
}

impl<T> AgentSessionServiceInterface for T where
    T: AgentSessionManager + DelegationTracker + CheckpointManager + Send + Sync
{
}

// ============================================================================
// Browse and Highlight Service Ports
// ============================================================================

/// Error type for browse operations
#[derive(Error, Debug)]
pub enum BrowseError {
    /// Specified path was not found
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    /// I/O error during browsing
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Highlighting failed for a file
    #[error("Highlighting failed: {0}")]
    HighlightingFailed(String),
}

/// Error type for highlighting operations
#[derive(Error, Debug)]
pub enum HighlightError {
    /// Invalid configuration for highlighting
    #[error("Highlighting configuration error: {0}")]
    ConfigurationError(String),

    /// Language is not supported by the highlighter
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Highlighting execution failed
    #[error("Highlighting failed: {0}")]
    HighlightingFailed(String),
}

/// Browse service trait (agnóstico interface)
#[async_trait]
pub trait BrowseServiceInterface: Send + Sync {
    /// Get file tree from given root path
    async fn get_file_tree(&self, root: &Path, max_depth: usize) -> Result<FileNode>;

    /// Get raw code from file
    async fn get_code(&self, path: &Path) -> Result<String>;

    /// Highlight code with given language
    async fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode>;

    /// Get code with highlighting
    async fn get_highlighted_code(&self, path: &Path) -> Result<HighlightedCode>;
}

/// Highlight service trait (agnóstico interface)
#[async_trait]
pub trait HighlightServiceInterface: Send + Sync {
    /// Highlight code with given language
    ///
    /// Returns structured highlight spans with byte offsets.
    /// Falls back to empty spans if highlighting fails.
    async fn highlight(&self, code: &str, language: &str) -> Result<HighlightedCode>;
}

// ============================================================================
// Chunking Service Port
// ============================================================================

/// Options for chunking operations
#[derive(Debug, Clone, Copy)]
pub struct ChunkingOptions {
    /// Maximum size of a single chunk in characters
    pub max_chunk_size: usize,
    /// Whether to include surrounding context (imports, class declarations, etc.)
    pub include_context: bool,
    /// Maximum number of chunks per file
    pub max_chunks_per_file: usize,
}

impl Default for ChunkingOptions {
    fn default() -> Self {
        Self {
            max_chunk_size: 512,
            include_context: true,
            max_chunks_per_file: 50,
        }
    }
}

/// Result of chunking a single file
#[derive(Debug, Clone)]
pub struct ChunkingResult {
    /// File path that was chunked
    pub file_path: String,
    /// Language detected for the file
    pub language: Language,
    /// Extracted chunks
    pub chunks: Vec<CodeChunk>,
    /// Whether AST parsing was successful (vs fallback)
    pub used_ast: bool,
}

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

/// Domain Port for Code Chunking Operations
#[async_trait]
pub trait CodeChunker: Send + Sync {
    /// Performs the chunk file operation.
    async fn chunk_file(
        &self,
        file_path: &Path,
        options: ChunkingOptions,
    ) -> Result<ChunkingResult>;

    /// Performs the chunk content operation.
    async fn chunk_content(
        &self,
        content: &str,
        file_name: &str,
        language: Language,
        options: ChunkingOptions,
    ) -> Result<ChunkingResult>;

    /// Performs the chunk batch operation.
    async fn chunk_batch(
        &self,
        file_paths: &[&Path],
        options: ChunkingOptions,
    ) -> Result<Vec<ChunkingResult>>;

    /// Performs the supported languages operation.
    fn supported_languages(&self) -> Vec<Language>;

    /// Performs the is language supported operation.
    fn is_language_supported(&self, language: &Language) -> bool {
        self.supported_languages().contains(language)
    }
}

// ============================================================================
// Context Service Port
// ============================================================================

/// Code Intelligence Service Interface
///
/// Defines the contract for semantic code understanding operations.
#[async_trait]
pub trait ContextServiceInterface: Send + Sync {
    /// Initialize the service for a collection
    async fn initialize(&self, collection: &CollectionId) -> Result<()>;

    /// Store code chunks in the repository
    async fn store_chunks(&self, collection: &CollectionId, chunks: &[CodeChunk]) -> Result<()>;

    /// Search for code similar to the query
    async fn search_similar(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Get embedding for text
    async fn embed_text(&self, text: &str) -> Result<Embedding>;

    /// Clear/delete a collection
    async fn clear_collection(&self, collection: &CollectionId) -> Result<()>;

    /// Get combined stats for the service
    async fn get_stats(&self) -> Result<(i64, i64)>;

    /// Get embedding dimensions
    fn embedding_dimensions(&self) -> usize;
}

// ============================================================================
// Hash Service Port
// ============================================================================

/// File hash state management port
#[async_trait]
pub trait FileHashService: Send + Sync {
    /// Check whether the file hash differs from what is stored or if it's new
    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool>;

    /// Insert or update the stored hash for a file
    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()>;

    /// List all files currently tracked in a collection
    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>>;

    /// Mark a file as deleted so it will be re-indexed later
    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()>;

    /// Compute the hash value for a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    fn compute_hash(path: &Path) -> Result<String>;
}

// ============================================================================
// Indexing Service Port
// ============================================================================

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

// ============================================================================
// Job Manager Interface
// ============================================================================

/// Unique identifier for a job (wraps `OperationId` for domain consistency)
pub type JobId = OperationId;

/// The type of work a job performs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display)]
pub enum JobType {
    /// Codebase indexing operation
    #[display("indexing")]
    Indexing,
    /// Architectural validation operation
    #[display("validation")]
    Validation,
    /// Code analysis / complexity assessment
    #[display("analysis")]
    Analysis,
    /// Custom job type with a user-defined label
    #[display("custom:{_0}")]
    Custom(String),
}

/// Lifecycle status of a job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    /// Job is waiting to be picked up
    Queued,
    /// Job is currently executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job terminated with an error
    Failed(String),
    /// Job was manually cancelled
    Cancelled,
}

impl JobStatus {
    /// Returns `true` if the job is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed(_) | Self::Cancelled)
    }

    /// Returns `true` if the job is actively running
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

/// Result metadata attached to a completed job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Summary message describing the outcome
    pub summary: String,
    /// Number of items successfully processed
    pub items_processed: usize,
    /// Number of items that failed processing
    pub items_failed: usize,
    /// Arbitrary key/value metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A generic background job tracked by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for the job
    pub id: JobId,
    /// What kind of work this job performs
    pub job_type: JobType,
    /// Human-readable label for the job
    pub label: String,
    /// Current lifecycle status
    pub status: JobStatus,
    /// Progress as a percentage (0..=100)
    pub progress_percent: u8,
    /// Number of items processed so far
    pub processed_items: usize,
    /// Total number of items to process (0 = unknown)
    pub total_items: usize,
    /// Description of the item currently being processed
    pub current_item: Option<String>,
    /// When the job was created/queued (Unix epoch seconds)
    pub created_at: i64,
    /// When the job started running (Unix epoch seconds, if applicable)
    pub started_at: Option<i64>,
    /// When the job reached a terminal state (Unix epoch seconds, if applicable)
    pub completed_at: Option<i64>,
    /// Result metadata (populated on completion)
    pub result: Option<JobResult>,
}

impl Job {
    /// Create a new job in `Queued` status
    pub fn new(id: JobId, job_type: JobType, label: impl Into<String>) -> Self {
        Self {
            id,
            job_type,
            label: label.into(),
            status: JobStatus::Queued,
            progress_percent: 0,
            processed_items: 0,
            total_items: 0,
            current_item: None,
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            result: None,
        }
    }
}

/// Progress update payload for advancing a running job
#[derive(Debug, Clone)]
pub struct JobProgressUpdate {
    /// Description of the current item being processed
    pub current_item: Option<String>,
    /// Number of items processed so far
    pub processed_items: usize,
    /// Total number of items to process
    pub total_items: usize,
}

/// Interface for managing the lifecycle of background jobs.
///
/// Implementations track creation, progress, completion, and cancellation
/// of jobs across all job types.
pub trait JobManagerInterface: Send + Sync {
    /// List all tracked jobs, optionally filtered by type
    fn list_jobs(&self, job_type: Option<&JobType>) -> Vec<Job>;

    /// Get a specific job by ID
    fn get_job(&self, job_id: &JobId) -> Option<Job>;

    /// Submit a new job and return its assigned ID
    fn submit_job(&self, job_type: JobType, label: &str, total_items: usize) -> JobId;

    /// Mark a queued job as running
    fn start_job(&self, job_id: &JobId);

    /// Update progress on a running job
    fn update_progress(&self, job_id: &JobId, update: JobProgressUpdate);

    /// Mark a job as successfully completed
    fn complete_job(&self, job_id: &JobId, result: Option<JobResult>);

    /// Mark a job as failed with an error message
    fn fail_job(&self, job_id: &JobId, error: &str);

    /// Cancel a queued or running job
    fn cancel_job(&self, job_id: &JobId);

    /// Get counts of jobs by status (for dashboard summaries)
    fn job_counts(&self) -> JobCounts;
}

/// Summary counts of jobs grouped by status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobCounts {
    /// Number of jobs waiting to start
    pub queued: usize,
    /// Number of actively running jobs
    pub running: usize,
    /// Number of successfully completed jobs
    pub completed: usize,
    /// Number of failed jobs
    pub failed: usize,
    /// Number of cancelled jobs
    pub cancelled: usize,
}

// ============================================================================
// Memory Service Port
// ============================================================================

/// Input payload for creating or updating a session summary.
#[derive(Debug, Clone)]
pub struct CreateSessionSummaryInput {
    /// Project identifier owning this session summary.
    pub project_id: String,
    /// Organization identifier owning this session summary.
    pub org_id: String,
    /// Session identifier being summarized.
    pub session_id: SessionId,
    /// Main topics covered in the session.
    pub topics: Vec<String>,
    /// Concrete decisions taken during the session.
    pub decisions: Vec<String>,
    /// Actionable next steps produced by the session.
    pub next_steps: Vec<String>,
    /// Important files touched or discussed.
    pub key_files: Vec<String>,
    /// Optional origin context metadata.
    pub origin_context: Option<OriginContext>,
}

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
        project_id: String,
        content: String,
        r#type: ObservationType,
        tags: Vec<String>,
        metadata: ObservationMetadata,
    ) -> Result<(ObservationId, bool)>;

    /// Store an error pattern.
    async fn store_error_pattern(&self, pattern: ErrorPattern) -> Result<String>;

    /// Search for error patterns.
    async fn search_error_patterns(
        &self,
        query: &str,
        project_id: String,
        limit: usize,
    ) -> Result<Vec<ErrorPattern>>;

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
    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>>;

    /// Create or update a session summary.
    ///
    /// Summarizes the key topics, decisions, and next steps from a session.
    async fn create_session_summary(&self, input: CreateSessionSummaryInput) -> Result<String>;

    /// Get an observation by ID.
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>>;

    /// Performs the delete observation operation.
    async fn delete_observation(&self, id: &ObservationId) -> Result<()>;

    /// Generate embedding for content (for external use).
    async fn embed_content(&self, content: &str) -> Result<Embedding>;

    /// Get observations in timeline order around an anchor (for progressive disclosure).
    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>>;

    /// Get multiple observations by IDs (for progressive disclosure step 3).
    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>>;

    /// Token-efficient memory search - returns index only (no full content).
    ///
    /// This is Step 1 of the 3-layer workflow (search -> timeline -> details).
    /// Returns lightweight index entries with IDs, types, tags, scores, and brief previews.
    /// Use memory action=get with the returned IDs for full details.
    async fn memory_search(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>>;
}

// ============================================================================
// Project Service Port
// ============================================================================

/// Defines behavior for `ProjectDetectorService`.
#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    /// Performs the detect all operation.
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}

// ============================================================================
// Search Service Port
// ============================================================================

/// Search Service Interface
///
/// Provides semantic code search capabilities.
#[async_trait]
pub trait SearchServiceInterface: Send + Sync {
    /// Search for code similar to the query
    async fn search(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Search with optional filters for more refined results
    async fn search_with_filters(
        &self,
        collection: &CollectionId,
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
// Validation Service Port
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

/// A single violation entry.
///
/// # Code Smells
/// TODO(qlty): Found 16 lines of similar code with `crates/mcb-validate/src/generic_reporter.rs`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ViolationEntry {
    /// Unique violation ID (e.g., "CA001", "SOLID002")
    pub id: String,
    /// Category (e.g., "`clean_architecture`", "solid", "quality")
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

/// Information about a validation rule
#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleInfo {
    /// Rule ID (e.g., "CA001")
    pub id: String,
    /// Rule category
    pub category: String,
    /// Rule severity (error, warning, info)
    pub severity: String,
    /// Human-readable description
    pub description: String,
    /// Engine that executes this rule
    pub engine: String,
}

/// Code complexity metrics report
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplexityReport {
    /// File path
    pub file: String,
    /// Cyclomatic complexity
    pub cyclomatic: f64,
    /// Cognitive complexity
    pub cognitive: f64,
    /// Maintainability index (0-100)
    pub maintainability_index: f64,
    /// Source lines of code
    pub sloc: usize,
    /// Function-level metrics (if requested)
    pub functions: Vec<FunctionComplexity>,
}

/// Function-level complexity metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct FunctionComplexity {
    /// Function name
    pub name: String,
    /// Start line number
    pub line: usize,
    /// Cyclomatic complexity
    pub cyclomatic: f64,
    /// Cognitive complexity
    pub cognitive: f64,
    /// Source lines of code
    pub sloc: usize,
}

/// Architecture Validation Service Interface
#[async_trait]
pub trait ValidationServiceInterface: Send + Sync {
    /// Performs the validate operation.
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport>;

    /// Performs the list validators operation.
    async fn list_validators(&self) -> Result<Vec<String>>;

    /// Performs the validate file operation.
    async fn validate_file(
        &self,
        file_path: &Path,
        validators: Option<&[String]>,
    ) -> Result<ValidationReport>;

    /// Performs the get rules operation.
    async fn get_rules(&self, category: Option<&str>) -> Result<Vec<RuleInfo>>;

    /// Performs the analyze complexity operation.
    async fn analyze_complexity(
        &self,
        file_path: &Path,
        include_functions: bool,
    ) -> Result<ComplexityReport>;
}
