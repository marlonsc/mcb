//! Hybrid search provider ports.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::SearchResult;

/// Result of a hybrid search operation.
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// The base search result metadata.
    pub result: SearchResult,
    /// BM25 keyword matching score.
    pub bm25_score: f32,
    /// Vector similarity score.
    pub semantic_score: f32,
    /// Fused score using Reciprocal Rank Fusion or similar.
    pub hybrid_score: f32,
}

/// Port for hybrid search operations.
#[async_trait]
pub trait HybridSearchProvider: Send + Sync {
    /// Index multiple code chunks in the search collection.
    ///
    /// # Errors
    /// Returns an error if indexing fails.
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;

    /// Perform a hybrid search combining keyword and semantic matching.
    ///
    /// # Errors
    /// Returns an error if search fails.
    async fn search(
        &self,
        collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Clear all data in the search collection.
    ///
    /// # Errors
    /// Returns an error if deletion fails.
    async fn clear_collection(&self, collection: &str) -> Result<()>;

    /// Get usage statistics for the hybrid search provider.
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}
