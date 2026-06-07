//! Hybrid search operations combining FTS and vector similarity.
//!
//! Implements Reciprocal Rank Fusion (RRF) to merge lexical and semantic results.

use std::collections::HashMap;
use std::str::FromStr;

use mcb_domain::constants::search::{
    HYBRID_SEARCH_MULTIPLIER, RRF_K, RRF_MAX_SCORE_STREAMS, RRF_NORMALIZED_MAX, RRF_SCORE_NUMERATOR,
};
use mcb_domain::entities::memory::{
    MemoryFilter, MemorySearchIndex, MemorySearchResult, Observation,
};
use mcb_domain::error::Result;
use mcb_domain::utils::id;
use mcb_domain::utils::id::compute_content_hash;
use mcb_domain::value_objects::{CollectionId, ObservationId};

use crate::constants::use_cases::OBSERVATION_PREVIEW_LENGTH;

use super::MemoryServiceImpl;

impl MemoryServiceImpl {
    /// Search memories using hybrid FTS + vector search with RRF ranking.
    pub(crate) async fn search_memories_impl(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        let candidate_limit = limit * HYBRID_SEARCH_MULTIPLIER;

        let query_embedding = self.embedding_provider.embed(query).await?;
        let collection_id = CollectionId::from_uuid(id::deterministic(
            "collection",
            crate::constants::use_cases::MEMORY_COLLECTION_NAME,
        ));

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
        let vector_results = match vector_result {
            Ok(results) => results,
            Err(e) => {
                mcb_domain::warn!(
                    "memory",
                    "Vector search failed — falling back to FTS-only results",
                    &format!("{e}")
                );
                Vec::new()
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

    /// Build ranked results from RRF scores, applying filters and fetching full observations.
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

    /// Build a memory search index from search results with content previews.
    pub(crate) fn build_memory_index(results: Vec<MemorySearchResult>) -> Vec<MemorySearchIndex> {
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
}
