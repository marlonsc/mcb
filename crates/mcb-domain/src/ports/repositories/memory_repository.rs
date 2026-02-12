//! Memory repository port for observation storage.

use async_trait::async_trait;

use crate::entities::memory::{MemoryFilter, Observation, SessionSummary};
use crate::error::Result;
use crate::value_objects::ids::{ObservationId, SessionId};

/// FTS search result with BM25 rank score
#[derive(Debug, Clone)]
pub struct FtsSearchResult {
    /// Observation ID
    pub id: String,
    /// BM25 rank score (lower is better, typically negative)
    pub rank: f64,
}

/// Port for observation storage (CRUD, FTS, timeline).
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    /// Performs the store observation operation.
    async fn store_observation(&self, observation: &Observation) -> Result<()>;
    /// Performs the get observation operation.
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>>;
    /// Performs the find by hash operation.
    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>>;

    /// Full-text search returning IDs with BM25 rank scores for hybrid fusion
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>>;

    /// Performs the delete observation operation.
    async fn delete_observation(&self, id: &ObservationId) -> Result<()>;

    /// Get multiple observations by IDs (batch fetch for hybrid search)
    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>>;

    /// Get observations in timeline order around an anchor
    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>>;

    /// Performs the store session summary operation.
    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()>;
    /// Performs the get session summary operation.
    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>>;
}
