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
use mcb_domain::ports::providers::VectorStoreProvider;
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::utils::compute_content_hash;
use mcb_domain::value_objects::Embedding;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const RRF_K: f32 = 60.0;
const HYBRID_SEARCH_MULTIPLIER: usize = 3;

pub struct MemoryServiceImpl {
    repository: Arc<dyn MemoryRepository>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl MemoryServiceImpl {
    pub fn new(
        repository: Arc<dyn MemoryRepository>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            repository,
            embedding_provider,
            vector_store,
        }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    fn matches_filter(obs: &Observation, filter: &MemoryFilter) -> bool {
        if let Some(ref session_id) = filter.session_id
            && obs.metadata.session_id.as_ref() != Some(session_id)
        {
            return false;
        }
        if let Some(ref repo_id) = filter.repo_id
            && obs.metadata.repo_id.as_ref() != Some(repo_id)
        {
            return false;
        }
        if let Some(ref obs_type) = filter.observation_type
            && &obs.observation_type != obs_type
        {
            return false;
        }
        if let Some((start, end)) = filter.time_range
            && (obs.created_at < start || obs.created_at > end)
        {
            return false;
        }
        true
    }
}

#[async_trait::async_trait]
impl MemoryServiceInterface for MemoryServiceImpl {
    async fn store_observation(
        &self,
        content: String,
        observation_type: ObservationType,
        tags: Vec<String>,
        metadata: ObservationMetadata,
    ) -> Result<String> {
        let content_hash = compute_content_hash(&content);

        if let Some(existing) = self.repository.find_by_hash(&content_hash).await? {
            return Ok(existing.id);
        }

        // Store embedding in vector store for hybrid search
        let embedding = self.embedding_provider.embed(&content).await?;

        let mut vector_metadata = std::collections::HashMap::new();
        vector_metadata.insert(
            "content".to_string(),
            serde_json::Value::String(content.clone()),
        );
        vector_metadata.insert(
            "type".to_string(),
            serde_json::Value::String(observation_type.as_str().to_string()),
        );
        vector_metadata.insert("tags".to_string(), serde_json::json!(tags));

        if let Some(session_id) = &metadata.session_id {
            vector_metadata.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.clone()),
            );
        }

        let ids = self
            .vector_store
            .insert_vectors("memories", &[embedding], vec![vector_metadata])
            .await?;

        let embedding_id = ids.first().cloned();

        let observation = Observation {
            id: Uuid::new_v4().to_string(),
            content,
            content_hash,
            tags,
            observation_type,
            metadata,
            created_at: Self::current_timestamp(),
            embedding_id,
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
        let candidate_limit = limit * HYBRID_SEARCH_MULTIPLIER;

        let query_embedding = self.embedding_provider.embed(query).await?;

        let fts_results = self
            .repository
            .search_fts_ranked(query, candidate_limit)
            .await?;

        let vector_results = self
            .vector_store
            .search_similar(
                "memories",
                query_embedding.vector.as_slice(),
                candidate_limit,
                None,
            )
            .await
            .unwrap_or_default();

        let mut rrf_scores: HashMap<String, f32> = HashMap::new();

        for (rank, fts_result) in fts_results.iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f32 + 1.0);
            *rrf_scores.entry(fts_result.id.clone()).or_default() += score;
        }

        for (rank, vec_result) in vector_results.iter().enumerate() {
            let content_hash = compute_content_hash(&vec_result.content);
            if let Ok(Some(obs)) = self.repository.find_by_hash(&content_hash).await {
                let score = 1.0 / (RRF_K + rank as f32 + 1.0);
                *rrf_scores.entry(obs.id.clone()).or_default() += score;
            }
        }

        let mut ranked: Vec<(String, f32)> = rrf_scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit);

        let top_ids: Vec<String> = ranked.iter().map(|(id, _)| id.clone()).collect();
        let observations = self.repository.get_observations_by_ids(&top_ids).await?;

        let obs_map: HashMap<String, Observation> = observations
            .into_iter()
            .map(|obs| (obs.id.clone(), obs))
            .collect();

        let filter = filter.unwrap_or_default();
        let mut results = Vec::new();
        for (id, rrf_score) in ranked {
            if let Some(obs) = obs_map.get(&id) {
                if !Self::matches_filter(obs, &filter) {
                    continue;
                }
                let max_possible_rrf = 2.0 / (RRF_K + 1.0);
                let normalized_score = (rrf_score / max_possible_rrf).min(1.0);
                results.push(MemorySearchResult {
                    observation: obs.clone(),
                    similarity_score: normalized_score,
                });
            }
        }

        Ok(results)
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

    async fn get_timeline(
        &self,
        anchor_id: &str,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        self.repository
            .get_timeline(anchor_id, before, after, filter)
            .await
    }

    async fn get_observations_by_ids(&self, ids: &[String]) -> Result<Vec<Observation>> {
        self.repository.get_observations_by_ids(ids).await
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
