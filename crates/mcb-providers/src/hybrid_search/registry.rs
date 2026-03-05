//! Hybrid search provider factory and auto-registration.

use std::sync::Arc;

use mcb_domain::ports::HybridSearchProvider as HybridSearchProviderPort;
use mcb_domain::registry::hybrid_search::HybridSearchProviderConfig;

use super::HybridSearchEngine;

/// Factory function for creating `HybridSearchEngine` instances.
fn hybrid_search_factory(
    _config: &HybridSearchProviderConfig,
) -> std::result::Result<Arc<dyn HybridSearchProviderPort>, String> {
    Ok(Arc::new(HybridSearchEngine::new()))
}

mcb_domain::register_hybrid_search_provider!(
    mcb_utils::constants::DEFAULT_HYBRID_SEARCH_PROVIDER,
    "Hybrid BM25 + semantic search engine (default)",
    hybrid_search_factory
);
