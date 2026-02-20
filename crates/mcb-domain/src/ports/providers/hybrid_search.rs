//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#provider-ports)
//!
//! Hybrid Search Port
//!
//! Defines the interface for hybrid search capabilities that combine
//! lexical (BM25) and semantic (vector) search.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::SearchResult;

/// Result of a hybrid search operation
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// The underlying search result with code chunk and metadata
    pub result: SearchResult,
    /// BM25 lexical matching score (0.0 to 1.0)
    pub bm25_score: f32,
    /// Semantic similarity score from vector search (0.0 to 1.0)
    pub semantic_score: f32,
    /// Combined hybrid score from both BM25 and semantic components
    pub hybrid_score: f32,
}

/// Port for hybrid search operations
///
/// Combines lexical (BM25) and semantic (vector) search for improved relevance.
/// BM25 excels at exact keyword matching while semantic search understands meaning.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::HybridSearchProvider;
/// use std::sync::Arc;
///
/// async fn search_hybrid(provider: Arc<dyn HybridSearchProvider>) -> mcb_domain::Result<()> {
///     // Perform hybrid search (requires semantic results from vector store)
///     let semantic_results = vec![];  // From vector store search
///     let results = provider.search("project", "async fn", semantic_results, 10).await?;
///
///     // Results are ranked by combined BM25 + semantic scores
///     for result in results {
///         println!("{}: score={:.3}", result.file_path, result.score);
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait HybridSearchProvider: Send + Sync {
    /// Index code chunks for hybrid search
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;

    /// Perform hybrid search
    async fn search(
        &self,
        collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Clear indexed data for a collection
    async fn clear_collection(&self, collection: &str) -> Result<()>;

    /// Get hybrid search statistics
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}
