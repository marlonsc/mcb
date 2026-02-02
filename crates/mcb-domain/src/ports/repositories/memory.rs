//! Memory repository port for observation storage.

use crate::entities::memory::{MemoryFilter, MemorySearchResult, Observation, SessionSummary};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait MemoryRepository: Send + Sync {
    async fn store_observation(&self, observation: &Observation) -> Result<()>;
    async fn get_observation(&self, id: &str) -> Result<Option<Observation>>;
    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>>;
    async fn search(
        &self,
        query_embedding: &[f32],
        filter: MemoryFilter,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>>;
    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()>;
    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>>;
}
