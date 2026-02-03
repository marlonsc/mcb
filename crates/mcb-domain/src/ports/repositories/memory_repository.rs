//! Memory repository port for observation storage.

use crate::entities::memory::{MemoryFilter, MemorySearchResult, Observation, SessionSummary};
use crate::error::Result;
use async_trait::async_trait;

/// FTS search result with BM25 rank score
#[derive(Debug, Clone)]
pub struct FtsSearchResult {
    /// Observation ID
    pub id: String,
    /// BM25 rank score (lower is better, typically negative)
    pub rank: f64,
}

#[async_trait]
pub trait MemoryRepository: Send + Sync {
    async fn store_observation(&self, observation: &Observation) -> Result<()>;
    async fn get_observation(&self, id: &str) -> Result<Option<Observation>>;
    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>>;

    /// Full-text search returning IDs only (for backward compatibility)
    async fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<String>>;

    /// Full-text search returning IDs with BM25 rank scores for hybrid fusion
    async fn search_fts_ranked(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>>;

    async fn delete_observation(&self, id: &str) -> Result<()>;

    /// Search with embedding (deprecated: use hybrid search in service layer)
    async fn search(
        &self,
        query_embedding: &[f32],
        filter: MemoryFilter,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>>;

    /// Get multiple observations by IDs (batch fetch for hybrid search)
    async fn get_observations_by_ids(&self, ids: &[String]) -> Result<Vec<Observation>>;

    /// Get observations in timeline order around an anchor
    async fn get_timeline(
        &self,
        anchor_id: &str,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>>;

    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()>;
    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>>;
}
