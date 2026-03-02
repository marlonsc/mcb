//! `MemoryServiceInterface` trait implementation.
//!
//! Provides the public API for memory operations.

use std::str::FromStr;

use mcb_domain::entities::memory::{
    ErrorPattern, MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation, ObservationType,
};
use mcb_domain::error::Result;
use mcb_domain::ports::{CreateSessionSummaryInput, MemoryServiceInterface};
use mcb_domain::value_objects::{Embedding, ObservationId, SessionId};

use super::MemoryServiceImpl;

#[async_trait::async_trait]
impl MemoryServiceInterface for MemoryServiceImpl {
    /// # Errors
    ///
    /// Returns an error if embedding generation, vector storage, or repository persistence fails.
    async fn store_observation(
        &self,
        project_id: String,
        content: String,
        r#type: ObservationType,
        tags: Vec<String>,
        metadata: mcb_domain::entities::memory::ObservationMetadata,
    ) -> Result<(ObservationId, bool)> {
        let (id, new) = self
            .store_observation_impl(project_id, content, r#type, tags, metadata)
            .await?;
        let obs_id = ObservationId::from_str(&id)
            .map_err(|e| mcb_domain::error::Error::invalid_argument(e.to_string()))?;
        Ok((obs_id, new))
    }

    /// # Errors
    ///
    /// Returns an error if serialization or observation storage fails.
    async fn store_error_pattern(&self, pattern: ErrorPattern) -> Result<String> {
        let content = serde_json::to_string(&pattern)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;

        let metadata = mcb_domain::entities::memory::ObservationMetadata {
            id: mcb_domain::utils::id::generate().to_string(),
            ..Default::default()
        };

        let (id, _) = self
            .store_observation(
                pattern.project_id.clone(),
                content,
                ObservationType::Error,
                pattern.tags,
                metadata,
            )
            .await?;

        Ok(id.to_string())
    }

    /// # Errors
    ///
    /// Returns an error if the memory search fails.
    async fn search_error_patterns(
        &self,
        query: &str,
        project_id: String,
        limit: usize,
    ) -> Result<Vec<ErrorPattern>> {
        let filter = MemoryFilter {
            project_id: Some(project_id),
            r#type: Some(ObservationType::Error),
            ..Default::default()
        };

        let results = self.search_memories(query, Some(filter), limit).await?;

        let mut patterns = Vec::new();
        for res in results {
            if let Ok(pattern) = serde_json::from_str::<ErrorPattern>(&res.observation.content) {
                patterns.push(pattern);
            }
        }
        Ok(patterns)
    }

    /// # Errors
    ///
    /// Returns an error if the hybrid search (FTS or vector) fails.
    async fn search_memories(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        self.search_memories_impl(query, filter, limit).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn get_session_summary(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<mcb_domain::entities::memory::SessionSummary>> {
        self.repository.get_session_summary(session_id).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository fails to persist the summary.
    async fn create_session_summary(&self, input: CreateSessionSummaryInput) -> Result<String> {
        self.create_session_summary_impl(input).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        self.repository.get_observation(id).await
    }

    /// # Errors
    ///
    /// Returns an error if the embedding provider fails.
    async fn embed_content(&self, content: &str) -> Result<Embedding> {
        self.embedding_provider.embed(content).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository timeline query fails.
    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        self.get_timeline_impl(anchor_id, before, after, filter)
            .await
    }

    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
        self.get_observations_by_ids_impl(ids).await
    }

    /// # Errors
    ///
    /// Returns an error if the hybrid search fails.
    async fn memory_search(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>> {
        self.memory_search_impl(query, filter, limit).await
    }

    /// # Errors
    ///
    /// Returns an error if the repository fails to delete the observation.
    async fn delete_observation(&self, id: &ObservationId) -> Result<()> {
        self.repository.delete_observation(id).await
    }
}
