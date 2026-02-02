//! Memory Service Use Case
//!
//! Application service for observation storage and semantic memory search.

use crate::ports::EmbeddingProvider;
use crate::ports::services::MemoryServiceInterface;
use mcb_domain::entities::memory::{
    MemoryFilter, MemorySearchResult, Observation, ObservationMetadata, ObservationType,
    SessionSummary,
};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::utils::compute_content_hash;
use mcb_domain::value_objects::Embedding;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct MemoryServiceImpl {
    repository: Arc<dyn MemoryRepository>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl MemoryServiceImpl {
    pub fn new(
        repository: Arc<dyn MemoryRepository>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            repository,
            embedding_provider,
        }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

#[async_trait::async_trait]
impl MemoryServiceInterface for MemoryServiceImpl {
    async fn store_observation(
        &self,
        content: String,
        observation_type: ObservationType,
        tags: Vec<String>,
        session_id: Option<String>,
        repo_id: Option<String>,
        file_path: Option<String>,
        branch: Option<String>,
    ) -> Result<String> {
        let content_hash = compute_content_hash(&content);

        if let Some(existing) = self.repository.find_by_hash(&content_hash).await? {
            return Ok(existing.id);
        }

        // TODO(Phase-6): Store embedding in vector store for hybrid search
        let _embedding = self.embedding_provider.embed(&content).await?;
        let embedding_id = Some(Uuid::new_v4().to_string());

        let observation = Observation {
            id: Uuid::new_v4().to_string(),
            content,
            content_hash,
            tags,
            observation_type,
            metadata: ObservationMetadata {
                session_id,
                repo_id,
                file_path,
                branch,
            },
            created_at: Self::current_timestamp(),
            embedding_id: embedding_id.clone(),
        };

        self.repository.store_observation(&observation).await?;

        Ok(observation.id)
    }

    async fn search_memories(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        let query_embedding = self.embedding_provider.embed(query).await?;
        self.repository
            .search(
                query_embedding.vector.as_slice(),
                filter.unwrap_or_default(),
                limit,
            )
            .await
    }

    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>> {
        self.repository.get_session_summary(session_id).await
    }

    async fn create_session_summary(
        &self,
        session_id: String,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String> {
        let summary = SessionSummary {
            id: Uuid::new_v4().to_string(),
            session_id,
            topics,
            decisions,
            next_steps,
            key_files,
            created_at: Self::current_timestamp(),
        };

        self.repository.store_session_summary(&summary).await?;
        Ok(summary.id)
    }

    async fn get_observation(&self, id: &str) -> Result<Option<Observation>> {
        self.repository.get_observation(id).await
    }

    async fn embed_content(&self, content: &str) -> Result<Embedding> {
        self.embedding_provider.embed(content).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp_is_reasonable() {
        let ts = MemoryServiceImpl::current_timestamp();
        assert!(ts > 1_700_000_000, "Timestamp should be after 2023");
        assert!(ts < 2_000_000_000, "Timestamp should be before 2033");
    }
}
