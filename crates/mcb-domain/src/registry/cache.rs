//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Cache Provider Registry
//!
//! Auto-registration system for cache providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;

/// Configuration for cache provider creation
///
/// Contains all configuration options that a cache provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct CacheProviderConfig {
    /// Provider name (e.g., "moka", "redis", "null")
    pub provider: String,
    /// Connection URI (for distributed caches)
    pub uri: Option<String>,
    /// Maximum cache size (entries or bytes depending on provider)
    pub max_size: Option<usize>,
    /// Default TTL in seconds
    pub ttl_secs: Option<u64>,
    /// Namespace prefix for keys
    pub namespace: Option<String>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(CacheProviderConfig {
    /// Set the connection URI (for distributed caches)
    uri: with_uri(into String),
    /// Set the maximum cache size (entries or bytes depending on provider)
    max_size: with_max_size(usize),
    /// Set the default TTL in seconds
    ttl_secs: with_ttl_secs(u64),
    /// Set the namespace prefix for keys
    namespace: with_namespace(into String),
});

crate::impl_registry!(
    provider_trait: crate::ports::CacheProvider,
    config_type: CacheProviderConfig,
    entry_type: CacheProviderEntry,
    slice_name: CACHE_PROVIDERS,
    resolve_fn: resolve_cache_provider,
    list_fn: list_cache_providers
);
