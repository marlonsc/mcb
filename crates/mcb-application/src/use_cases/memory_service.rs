//! Memory Service Use Case
//!
//! Application service for observation storage and semantic memory search.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::entities::memory::{
    ErrorPattern, MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation,
    ObservationMetadata, ObservationType, SessionSummary,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::ports::providers::VectorStoreProvider;
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::compute_content_hash;
use mcb_domain::value_objects::{CollectionId, Embedding, ObservationId, SessionId};
use uuid::Uuid;

use crate::constants::{
    HYBRID_SEARCH_MULTIPLIER, MEMORY_COLLECTION_NAME, OBSERVATION_PREVIEW_LENGTH, RRF_K,
};

/// Hybrid memory service: SQLite for metadata/FTS + VectorStore for RAG embeddings.
pub struct MemoryServiceImpl {
    project_id: String,
    repository: Arc<dyn MemoryRepository>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl MemoryServiceImpl {
    /// Initializes the hybrid memory service with repository, embedding, and vector store providers.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project identifier for scoping observations and memories.
    /// * `repository` - SQLite-backed repository for metadata storage and full-text search.
    /// * `embedding_provider` - Provider for generating vector embeddings from content.
    /// * `vector_store` - Vector store for semantic similarity search and RAG operations.
    ///
    /// The service implements a hybrid search strategy combining full-text search (FTS)
    /// with vector similarity using reciprocal rank fusion (RRF) for balanced relevance.
    pub fn new(
        project_id: String,
        repository: Arc<dyn MemoryRepository>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            project_id,
            repository,
            embedding_provider,
            vector_store,
        }
    }

    /// Returns the current Unix timestamp in seconds.
    ///
    /// Used to record observation creation times and session summary timestamps.
    /// Falls back to 0 if the system clock is unavailable (extremely rare).
    ///
    /// # Returns
    ///
    /// Current Unix timestamp as seconds since UNIX_EPOCH, or 0 if unavailable.
    #[must_use]
    pub fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    fn matches_filter(obs: &Observation, filter: &MemoryFilter) -> bool {
        if !Self::check_project(obs, filter) {
            return false;
        }
        if !Self::check_session(obs, filter) {
            return false;
        }
        if !Self::check_repo(obs, filter) {
            return false;
        }
        if !Self::check_type(obs, filter) {
            return false;
        }
        if !Self::check_time(obs, filter) {
            return false;
        }
        if !Self::check_branch(obs, filter) {
            return false;
        }
        Self::check_commit(obs, filter)
    }

    fn check_project(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .project_id
            .as_ref()
            .is_none_or(|id| obs.project_id == *id)
    }

    fn check_session(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .session_id
            .as_ref()
            .is_none_or(|id| obs.metadata.session_id.as_ref() == Some(id))
    }

    fn check_repo(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .repo_id
            .as_ref()
            .is_none_or(|id| obs.metadata.repo_id.as_ref() == Some(id))
    }

    fn check_type(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .r#type
            .as_ref()
            .is_none_or(|t| &obs.r#type == t)
    }

    fn check_time(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .time_range
            .as_ref()
            .is_none_or(|(start, end)| obs.created_at >= *start && obs.created_at <= *end)
    }

    fn check_branch(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .branch
            .as_ref()
            .is_none_or(|b| obs.metadata.branch.as_ref() == Some(b))
    }

    fn check_commit(obs: &Observation, filter: &MemoryFilter) -> bool {
        filter
            .commit
            .as_ref()
            .is_none_or(|c| obs.metadata.commit.as_ref() == Some(c))
    }
}

impl MemoryServiceImpl {
    async fn store_observation_impl(
        &self,
        project_id: String,
        content: String,
        r#type: ObservationType,
        tags: Vec<String>,
        metadata: ObservationMetadata,
    ) -> Result<(String, bool)> {
        if project_id.trim().is_empty() {
            return Err(mcb_domain::error::Error::invalid_argument(
                "Project ID cannot be empty for memory storage",
            ));
        }

        let content_hash = compute_content_hash(&content);

        if let Some(existing) = self.repository.find_by_hash(&content_hash).await? {
            return Ok((existing.id, true));
        }

        let embedding = self.embedding_provider.embed(&content).await?;

        let mut vector_metadata = HashMap::new();
        vector_metadata.insert(
            "content".to_string(),
            serde_json::Value::String(content.clone()),
        );
        vector_metadata.insert(
            "type".to_string(),
            serde_json::Value::String(observation_type.as_str().to_string()),
        );
        vector_metadata.insert("tags".to_string(), serde_json::json!(tags));
        vector_metadata.insert(
            "project_id".to_string(),
            serde_json::Value::String(project_id.clone()),
        );

        if let Some(session_id) = &metadata.session_id {
            vector_metadata.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.clone()),
            );
        }

        let collection_id = CollectionId::new(crate::constants::MEMORY_COLLECTION_NAME);
        let ids = self
            .vector_store
            .insert_vectors(&collection_id, &[embedding], vec![vector_metadata])
            .await?;

        let embedding_id = ids.first().cloned();

        let observation = Observation {
            id: Uuid::new_v4().to_string(),
            project_id,
            content,
            content_hash,
            tags,
            observation_type,
            metadata,
            created_at: Self::current_timestamp(),
            embedding_id,
        };

        self.repository.store_observation(&observation).await?;

        Ok((observation.id, false))
    }
}

impl MemoryServiceImpl {
    async fn search_memories_impl(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        let candidate_limit = limit * HYBRID_SEARCH_MULTIPLIER;

        let query_embedding = self.embedding_provider.embed(query).await?;
        let collection_id = CollectionId::new(MEMORY_COLLECTION_NAME);

        let (fts_result, vector_result) = tokio::join!(
            self.repository.search(query, candidate_limit),
            self.vector_store.search_similar(
                &collection_id,
                query_embedding.vector.as_slice(),
                candidate_limit,
                None,
            ),
        );
        let fts_results = fts_result?;
        let vector_results = vector_result.unwrap_or_default();

        let mut rrf_scores: HashMap<String, f32> = HashMap::new();

        for (rank, fts_result) in fts_results.iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f32 + 1.0);
            let key = fts_result.id.clone();
            *rrf_scores.entry(key).or_default() += score;
        }

        for (rank, vec_result) in vector_results.iter().enumerate() {
            let content_hash = compute_content_hash(&vec_result.content);
            if let Ok(Some(obs)) = self.repository.find_by_hash(&content_hash).await {
                let score = 1.0 / (RRF_K + rank as f32 + 1.0);
                let key = obs.id.clone();
                *rrf_scores.entry(key).or_default() += score;
            }
        }

        let mut ranked: Vec<(String, f32)> = rrf_scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit);

        let applied_filter = filter.unwrap_or_default();
        self.build_ranked_results(ranked, &applied_filter).await
    }

    async fn build_ranked_results(
        &self,
        ranked: Vec<(String, f32)>,
        filter: &MemoryFilter,
    ) -> Result<Vec<MemorySearchResult>> {
        let top_ids: Vec<ObservationId> = ranked
            .iter()
            .map(|(id, _)| ObservationId::new(id))
            .collect();
        let observations = self.repository.get_observations_by_ids(&top_ids).await?;

        let obs_map: HashMap<String, Observation> = observations
            .into_iter()
            .map(|obs| (obs.id.clone(), obs))
            .collect();

        let mut results = Vec::new();
        for (id, rrf_score) in ranked {
            if let Some(obs) = obs_map.get(&id) {
                if !Self::matches_filter(obs, filter) {
                    continue;
                }
                let max_possible_rrf = 2.0 / (RRF_K + 1.0);
                let normalized_score = (rrf_score / max_possible_rrf).min(1.0);
                results.push(MemorySearchResult {
                    id: id.clone(),
                    observation: obs.clone(),
                    similarity_score: normalized_score,
                });
            }
        }

        Ok(results)
    }
}

impl MemoryServiceImpl {
    fn build_memory_index(&self, results: Vec<MemorySearchResult>) -> Vec<MemorySearchIndex> {
        results
            .into_iter()
            .map(|r| {
                let content_preview = if r.observation.content.len() > OBSERVATION_PREVIEW_LENGTH {
                    format!(
                        "{}...",
                        &r.observation.content[..OBSERVATION_PREVIEW_LENGTH]
                    )
                } else {
                    r.observation.content.clone()
                };

                MemorySearchIndex {
                    id: r.observation.id,
                    r#type: r.observation.r#type.as_str().to_string(),
                    relevance_score: r.similarity_score,
                    tags: r.observation.tags,
                    content_preview,
                    session_id: r.observation.metadata.session_id,
                    repo_id: r.observation.metadata.repo_id,
                    file_path: r.observation.metadata.file_path,
                    created_at: r.observation.created_at,
                }
            })
            .collect()
    }

    async fn create_session_summary_impl(
        &self,
        session_id: SessionId,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String> {
        let summary = SessionSummary {
            id: Uuid::new_v4().to_string(),
            project_id: self.project_id.clone(),
            session_id: session_id.into_string(),
            topics,
            decisions,
            next_steps,
            key_files,
            created_at: Self::current_timestamp(),
        };

        self.repository.store_session_summary(&summary).await?;
        Ok(summary.id)
    }

    async fn embed_content_impl(&self, content: &str) -> Result<Embedding> {
        self.embedding_provider.embed(content).await
    }

    async fn get_timeline_impl(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        self.repository
            .get_timeline(anchor_id, before, after, filter)
            .await
    }

    async fn get_observations_by_ids_impl(
        &self,
        ids: &[ObservationId],
    ) -> Result<Vec<Observation>> {
        self.repository.get_observations_by_ids(ids).await
    }

    async fn memory_search_impl(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>> {
        let results = self.search_memories_impl(query, filter, limit).await?;
        Ok(self.build_memory_index(results))
    }
}

#[async_trait::async_trait]
impl MemoryServiceInterface for MemoryServiceImpl {
    async fn store_observation(
        &self,
        project_id: String,
        content: String,
        r#type: ObservationType,
        tags: Vec<String>,
        metadata: ObservationMetadata,
    ) -> Result<(ObservationId, bool)> {
        let (id, new) = self
            .store_observation_impl(project_id, content, observation_type, tags, metadata)
            .await?;
        Ok((ObservationId::new(id), new))
    }

    async fn store_error_pattern(&self, pattern: ErrorPattern) -> Result<String> {
        let content = serde_json::to_string(&pattern)
            .map_err(|e| mcb_domain::error::Error::generic(e.to_string()))?;

        let metadata = ObservationMetadata {
            id: Uuid::new_v4().to_string(),
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

        Ok(id.into_string())
    }

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

    async fn search_memories(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        self.search_memories_impl(query, filter, limit).await
    }

    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
        self.repository.get_session_summary(session_id).await
    }

    async fn create_session_summary(
        &self,
        session_id: SessionId,
        topics: Vec<String>,
        decisions: Vec<String>,
        next_steps: Vec<String>,
        key_files: Vec<String>,
    ) -> Result<String> {
        self.create_session_summary_impl(session_id, topics, decisions, next_steps, key_files)
            .await
    }

    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        self.repository.get_observation(id).await
    }

    async fn embed_content(&self, content: &str) -> Result<Embedding> {
        self.embed_content_impl(content).await
    }

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

    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
        self.get_observations_by_ids_impl(ids).await
    }

    async fn memory_search(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchIndex>> {
        self.memory_search_impl(query, filter, limit).await
    }
}
