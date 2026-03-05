//! Memory service ports.

use async_trait::async_trait;

use crate::entities::memory::{
    ErrorPattern, MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation,
    ObservationMetadata, ObservationType, OriginContext, SessionSummary,
};
use crate::error::Result;
use crate::value_objects::{Embedding, ObservationId, SessionId};

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

/// Manager for core observations.
#[async_trait]
pub trait ObservationManager: Send + Sync {
    /// Store an observation with optional embedding for semantic search.
    async fn store_observation(
        &self,
        project_id: String,
        content: String,
        r#type: ObservationType,
        tags: Vec<String>,
        metadata: ObservationMetadata,
    ) -> Result<(ObservationId, bool)>;

    /// Get an observation by ID.
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>>;

    /// Performs the delete observation operation.
    async fn delete_observation(&self, id: &ObservationId) -> Result<()>;

    /// Get multiple observations by IDs.
    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>>;
}

/// Manager for error patterns.
#[async_trait]
pub trait ErrorPatternManager: Send + Sync {
    /// Store an error pattern.
    async fn store_error_pattern(&self, pattern: ErrorPattern) -> Result<String>;

    /// Search for error patterns.
    async fn search_error_patterns(
        &self,
        query: &str,
        project_id: String,
        limit: usize,
    ) -> Result<Vec<ErrorPattern>>;
}

/// Manager for session summaries.
#[async_trait]
pub trait SessionSummaryManager: Send + Sync {
    /// Get a session summary by session ID.
    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>>;

    /// Create or update a session summary.
    async fn create_session_summary(&self, input: CreateSessionSummaryInput) -> Result<String>;
}

/// Semantic text operations and memory search.
#[async_trait]
pub trait MemorySearcher: Send + Sync {
    /// Search memories using semantic similarity.
    async fn search_memories(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>>;

    /// Generate embedding for content (for external use).
    async fn embed_content(&self, content: &str) -> Result<Embedding>;

    /// Get observations in timeline order around an anchor.
    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>>;

    /// Token-efficient memory search - returns index only (no full content).
    async fn memory_search(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>>;
}

define_aggregate! {
    /// Memory Service Interface
    ///
    /// Provides observation storage and retrieval with semantic search capabilities.
    /// Supports session-based memory organization and content deduplication.
    #[async_trait]
    pub trait MemoryServiceInterface = ObservationManager + ErrorPatternManager + SessionSummaryManager + MemorySearcher;
}
