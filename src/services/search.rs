//! Search service for querying indexed code

use crate::core::error::Result;
use crate::core::types::SearchResult;
use crate::services::context::ContextService;

/// Simple search service for MVP
pub struct SearchService {
    context_service: std::sync::Arc<ContextService>,
}

impl SearchService {
    /// Create a new search service
    pub fn new(context_service: std::sync::Arc<ContextService>) -> Self {
        Self { context_service }
    }

    /// Search for code similar to the query
    pub async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.context_service
            .search_similar(collection, query, limit)
            .await
    }
}
