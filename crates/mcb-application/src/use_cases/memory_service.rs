//! Memory Service Use Case
//!
//! # Overview
//! The `MemoryService` implements a comprehensive system for storing, retrieving, and analyzing
//! observations and long-term memory. It acts as the "brain" of the system, allowing agents
//! to recall past context, decisions, and error patterns.
//!
//! # Responsibilities
//! - **Hybrid Storage**: Persisting observations in both a relational DB (`SQLite`) for metadata/FTS
//!   and a Vector Store for semantic similarity.
//! - **Hybrid Search**: Combining keyword-based (FTS) and semantic (Vector) search results using
//!   Reciprocal Rank Fusion (RRF) for high-quality recall.
//! - **Timeline Management**: Retrieving observations in chronological order to reconstruct context.
//! - **Pattern Recognition**: Storing and retrieving error patterns to avoid repeating mistakes.
//! - **Session Summarization**: Compiling and storing high-level summaries of agent sessions.
//!
//! # Architecture
//! Implements `MemoryServiceInterface` and coordinates:
//! - `MemoryRepository`: For precise storage and FTS.
//! - `VectorStoreProvider`: For fuzzy semantic search.
//! - `EmbeddingProvider`: For generating vector representations of memory content.

use std::collections::HashMap;
use std::sync::Arc;

use mcb_domain::constants::keys::{
    METADATA_KEY_CONTENT, METADATA_KEY_SESSION_ID, METADATA_KEY_TAGS, METADATA_KEY_TYPE,
};
use mcb_domain::entities::memory::{
    ErrorPattern, MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation,
    ObservationMetadata, ObservationType, OriginContext, SessionSummary,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::ports::providers::VectorStoreProvider;
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::ports::services::{CreateSessionSummaryInput, MemoryServiceInterface};
use mcb_domain::utils::compute_content_hash;
use mcb_domain::utils::id;
use mcb_domain::utils::time as domain_time;
use mcb_domain::value_objects::{CollectionId, Embedding, ObservationId, SessionId};
use std::str::FromStr;

use crate::constants::{
    HYBRID_SEARCH_MULTIPLIER, MEMORY_COLLECTION_NAME, OBSERVATION_PREVIEW_LENGTH, RRF_K,
    RRF_MAX_SCORE_STREAMS, RRF_NORMALIZED_MAX, RRF_SCORE_NUMERATOR,
};

/// Hybrid memory service combining relational metadata with semantic vector search.
///
/// Implements a sophisticated RAG (Retrieval-Augmented Generation) pipeline using
/// Reciprocal Rank Fusion (RRF) to merge lexically precise matches (`SQLite` FTS)
/// with semantically relevant results (Vector Store).
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
            METADATA_KEY_CONTENT.to_owned(),
            serde_json::Value::String(content.clone()),
        );
        vector_metadata.insert(
            METADATA_KEY_TYPE.to_owned(),
            serde_json::Value::String(r#type.as_str().to_owned()),
        );
        vector_metadata.insert(METADATA_KEY_TAGS.to_owned(), serde_json::json!(tags));
        vector_metadata.insert(
            "project_id".to_owned(),
            serde_json::Value::String(project_id.clone()),
        );

        if let Some(session_id) = &metadata.session_id {
            vector_metadata.insert(
                METADATA_KEY_SESSION_ID.to_owned(),
                serde_json::Value::String(session_id.clone()),
            );
        }

        let collection_id = CollectionId::from_uuid(id::deterministic(
            "collection",
            crate::constants::MEMORY_COLLECTION_NAME,
        ));
        let ids = self
            .vector_store
            .insert_vectors(&collection_id, &[embedding], vec![vector_metadata])
            .await?;

        let embedding_id = ids.first().cloned();

        let observation = Observation {
            id: id::generate().to_string(),
            project_id,
            content,
            content_hash,
            tags,
            r#type,
            metadata,
            created_at: domain_time::epoch_secs_i64()?,
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
        let collection_id =
            CollectionId::from_uuid(id::deterministic("collection", MEMORY_COLLECTION_NAME));

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
        let (vector_results, _vector_search_failed) = match vector_result {
            Ok(results) => (results, false),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Vector search failed — falling back to FTS-only results"
                );
                (Vec::new(), true)
            }
        };

        let mut rrf_scores: HashMap<String, f32> = HashMap::new();

        for (rank, fts_result) in fts_results.iter().enumerate() {
            let score = RRF_SCORE_NUMERATOR / (RRF_K + rank as f32 + 1.0);
            let key = fts_result.id.clone();
            *rrf_scores.entry(key).or_default() += score;
        }

        for (rank, vec_result) in vector_results.iter().enumerate() {
            let content_hash = compute_content_hash(&vec_result.content);
            if let Ok(Some(obs)) = self.repository.find_by_hash(&content_hash).await {
                let score = RRF_SCORE_NUMERATOR / (RRF_K + rank as f32 + 1.0);
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
            .filter_map(|(id, _)| ObservationId::from_str(id).ok())
            .collect();
        let observations = self.repository.get_observations_by_ids(&top_ids).await?;

        let obs_map: HashMap<String, Observation> = observations
            .into_iter()
            .map(|obs| (obs.id.clone(), obs))
            .collect();

        let mut results = Vec::new();
        for (id, rrf_score) in ranked {
            if let Some(obs) = obs_map.get(&id) {
                if !filter.matches(obs) {
                    continue;
                }
                let max_possible_rrf = RRF_MAX_SCORE_STREAMS / (RRF_K + 1.0);
                let normalized_score = (rrf_score / max_possible_rrf).min(RRF_NORMALIZED_MAX);
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
    fn build_memory_index(results: Vec<MemorySearchResult>) -> Vec<MemorySearchIndex> {
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
                    r#type: r.observation.r#type.as_str().to_owned(),
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
        input: CreateSessionSummaryInput,
    ) -> Result<String> {
        let session_id = input.session_id.to_string();
        let timestamp = domain_time::epoch_secs_i64()?;
        let project_id = if input.project_id.trim().is_empty() {
            self.project_id.clone()
        } else {
            input.project_id
        };
        let summary = SessionSummary {
            id: id::generate().to_string(),
            project_id: project_id.clone(),
            session_id: session_id.clone(),
            topics: input.topics,
            decisions: input.decisions,
            next_steps: input.next_steps,
            key_files: input.key_files,
            origin_context: Some(
                input.origin_context.unwrap_or(
                    OriginContext::builder()
                        .project_id(Some(project_id))
                        .session_id(Some(session_id))
                        .timestamp(Some(timestamp))
                        .build(),
                ),
            ),
            created_at: timestamp,
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
        Ok(Self::build_memory_index(results))
    }
}

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
        metadata: ObservationMetadata,
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

        let metadata = ObservationMetadata {
            id: id::generate().to_string(),
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
    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
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
        self.embed_content_impl(content).await
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
