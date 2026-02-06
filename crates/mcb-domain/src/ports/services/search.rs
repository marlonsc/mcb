use crate::error::Result;
use crate::value_objects::{CollectionId, SearchResult};
use async_trait::async_trait;

/// Search Service Interface
///
/// Provides semantic code search capabilities.
#[async_trait]
pub trait SearchServiceInterface: Send + Sync {
    /// Search for code similar to the query
    async fn search(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Search with optional filters for more refined results
    async fn search_with_filters(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
        filters: Option<&SearchFilters>,
    ) -> Result<Vec<SearchResult>>;
}

/// Filters for search queries
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Filter by file extension (e.g., "rs", "py")
    pub file_extensions: Option<Vec<String>>,
    /// Filter by programming language
    pub languages: Option<Vec<String>>,
    /// Minimum relevance score threshold (0.0 to 1.0)
    pub min_score: Option<f32>,
}
