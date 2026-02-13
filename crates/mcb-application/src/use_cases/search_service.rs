//! Search Service Use Case
//!
//! # Overview
//! The `SearchService` executes semantic search queries against indexed codebases.
//! It applies business logic like result ranking and post-filtering (e.g., by file type or language)
//! to refine the raw results from the `ContextService`.
//! This separation allows the search logic to evolve (e.g., hybrid search, re-ranking) without
//! complicating the core context management.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::services::{ContextServiceInterface, SearchFilters, SearchServiceInterface};
use mcb_domain::value_objects::{CollectionId, SearchResult};

/// Implementation of the `SearchServiceInterface`.
///
/// Orchestrates vector similarity search via `ContextService` and applies application-level
/// filtering logic.
pub struct SearchServiceImpl {
    context_service: Arc<dyn ContextServiceInterface>,
}

impl SearchServiceImpl {
    /// Create new search service with injected dependencies
    pub fn new(context_service: Arc<dyn ContextServiceInterface>) -> Self {
        Self { context_service }
    }

    /// Apply filters to search results
    // TODO(architecture): Push filters down to repository/vector store level for efficiency.
    // In-memory filtering after over-fetching (limit * 2) is simpler but scales poorly.
    fn apply_filters(
        results: Vec<SearchResult>,
        filters: Option<&SearchFilters>,
    ) -> Vec<SearchResult> {
        let Some(filters) = filters else {
            return results;
        };

        results
            .into_iter()
            .filter(|r| {
                // Filter by minimum score
                if let Some(min_score) = filters.min_score
                    && r.score < f64::from(min_score)
                {
                    return false;
                }

                // Filter by file extension
                if let Some(ref exts) = filters.file_extensions {
                    let file_ext = std::path::Path::new(&r.file_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if !exts.iter().any(|e| e == file_ext) {
                        return false;
                    }
                }

                // Filter by language
                if let Some(ref langs) = filters.languages
                    && !langs.iter().any(|l| l == &r.language)
                {
                    return false;
                }

                true
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl SearchServiceInterface for SearchServiceImpl {
    async fn search(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.context_service
            .search_similar(collection, query, limit)
            .await
    }

    async fn search_with_filters(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
        filters: Option<&SearchFilters>,
    ) -> Result<Vec<SearchResult>> {
        // Get more results initially to account for filtering
        let fetch_limit = if filters.is_some() { limit * 2 } else { limit };
        let results = self
            .context_service
            .search_similar(collection, query, fetch_limit)
            .await?;

        // Apply filters and limit
        let filtered = Self::apply_filters(results, filters);
        Ok(filtered.into_iter().take(limit).collect())
    }
}
