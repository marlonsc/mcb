//! Embedding Provider Registry
//!
//! Auto-registration system for embedding providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for embedding provider creation
///
/// Contains all configuration options that an embedding provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct EmbeddingProviderConfig {
    /// Provider name (e.g., "ollama", "openai", "null")
    pub provider: String,
    /// Model name/identifier
    pub model: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Base URL for the provider API
    pub base_url: Option<String>,
    /// Embedding dimensions (if configurable)
    pub dimensions: Option<usize>,
    /// Cache directory for local providers (`FastEmbed`)
    pub cache_dir: Option<PathBuf>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

impl EmbeddingProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set the model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL for the API
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the embedding dimensions
    #[must_use]
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Set the cache directory
    pub fn with_cache_dir(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

crate::impl_registry!(
    provider_trait: crate::ports::providers::EmbeddingProvider,
    config_type: EmbeddingProviderConfig,
    entry_type: EmbeddingProviderEntry,
    slice_name: EMBEDDING_PROVIDERS,
    resolve_fn: resolve_embedding_provider,
    list_fn: list_embedding_providers
);
