//! Hybrid search provider registry.
//!
//! Auto-registration for hybrid (BM25 + semantic) search providers via linkme.

use std::collections::HashMap;

/// Configuration for hybrid search provider resolution.
#[derive(Debug, Clone, Default)]
pub struct HybridSearchProviderConfig {
    /// Provider name (e.g. "default").
    pub provider: String,
    /// Additional provider-specific configuration.
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(HybridSearchProviderConfig {});

crate::impl_registry!(
    provider_trait: crate::ports::HybridSearchProvider,
    config_type: HybridSearchProviderConfig,
    entry_type: HybridSearchProviderEntry,
    slice_name: HYBRID_SEARCH_PROVIDERS,
    resolve_fn: resolve_hybrid_search_provider,
    list_fn: list_hybrid_search_providers
);
