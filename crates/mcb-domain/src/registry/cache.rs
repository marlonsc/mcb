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

impl CacheProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set the connection URI (for distributed caches)
    #[must_use]
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the maximum cache size (entries or bytes depending on provider)
    #[must_use]
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    /// Set the default TTL in seconds
    #[must_use]
    pub fn with_ttl_secs(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = Some(ttl_secs);
        self
    }

    /// Set the namespace prefix for keys
    #[must_use]
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Add extra configuration
    #[must_use]
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

crate::impl_registry!(
    provider_trait: crate::ports::providers::CacheProvider,
    config_type: CacheProviderConfig,
    entry_type: CacheProviderEntry,
    slice_name: CACHE_PROVIDERS,
    resolve_fn: resolve_cache_provider,
    list_fn: list_cache_providers
);
