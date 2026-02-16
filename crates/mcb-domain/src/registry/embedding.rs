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

crate::impl_config_builder!(EmbeddingProviderConfig {
    /// Set the model name
    model: with_model(into String),
    /// Set the API key
    api_key: with_api_key(into String),
    /// Set the base URL for the API
    base_url: with_base_url(into String),
    /// Set the embedding dimensions
    dimensions: with_dimensions(usize),
    /// Set the cache directory
    cache_dir: with_cache_dir(into PathBuf),
});

crate::impl_registry!(
    provider_trait: crate::ports::providers::EmbeddingProvider,
    config_type: EmbeddingProviderConfig,
    entry_type: EmbeddingProviderEntry,
    slice_name: EMBEDDING_PROVIDERS,
    resolve_fn: resolve_embedding_provider,
    list_fn: list_embedding_providers
);
