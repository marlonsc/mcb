//!
//! **Documentation**: [docs/modules/application.md](../../../../docs/modules/application.md#use-cases)
//!
//! Search Service Use Case
//!
//! # Overview
//! The `SearchService` executes semantic search queries against indexed codebases.
//! It applies business logic like result ranking and post-filtering (e.g., by file type or language)
//! to refine the raw results from the `ContextService`.
//! This separation allows the search logic to evolve (e.g., hybrid search, re-ranking) without
//! complicating the core context management.

use std::sync::Arc;

use mcb_domain::constants::search::SEARCH_OVERFETCH_MULTIPLIER;
use mcb_domain::error::Result;
use mcb_domain::ports::{ContextServiceInterface, SearchFilters, SearchServiceInterface};
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

    /// Apply filters to search results in-memory after retrieval.
    ///
    /// # Design Note
    /// Filters are applied in-memory after over-fetching (`limit * 2`) from the vector store.
    /// For large-scale deployments, push filters down to the vector store level via
    /// `ContextServiceInterface::search_similar` metadata filters parameter.
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
    /// # Errors
    ///
    /// Returns an error if the context service search fails.
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

    /// # Errors
    ///
    /// Returns an error if the context service search fails.
    async fn search_with_filters(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
        filters: Option<&SearchFilters>,
    ) -> Result<Vec<SearchResult>> {
        // Get more results initially to account for filtering
        let fetch_limit = if filters.is_some() {
            limit * SEARCH_OVERFETCH_MULTIPLIER
        } else {
            limit
        };
        let results = self
            .context_service
            .search_similar(collection, query, fetch_limit)
            .await?;

        // Apply filters and limit
        let filtered = Self::apply_filters(results, filters);
        Ok(filtered.into_iter().take(limit).collect())
    }
}

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::services::{
    SEARCH_SERVICE_NAME, SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry,
};

// linkme distributed_slice uses #[link_section] internally
#[allow(unsafe_code)]
#[linkme::distributed_slice(SERVICES_REGISTRY)]
static SEARCH_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: SEARCH_SERVICE_NAME,
    build: ServiceBuilder::Search(|context| {
        let context_service = mcb_domain::registry::services::resolve_context_service(context)?;
        Ok(Arc::new(SearchServiceImpl::new(context_service)))
    }),
};
