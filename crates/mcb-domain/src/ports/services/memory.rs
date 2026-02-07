use async_trait::async_trait;

use crate::entities::memory::{
    MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation, ObservationMetadata,
    ObservationType, SessionSummary,
};
use crate::error::Result;
use crate::value_objects::{Embedding, ObservationId, SessionId};

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
        metadata: ObservationMetadata,
    ) -> Result<(ObservationId, bool)>;

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
    async fn create_session_summary(
        &self,
        session_id: SessionId,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String>;

    /// Get an observation by ID.
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>>;

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
