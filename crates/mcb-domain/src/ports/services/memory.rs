//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#service-ports)
//!
//! Memory Service Port
//!
//! # Overview
//! Defines the interface for managing long-term agent memory, including semantic search,
//! error patterns, and session summaries.
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
