//! Vector Store Provider Registry
//!
//! Auto-registration system for vector store providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;

/// Configuration for vector store provider creation
///
/// Contains all configuration options that a vector store provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct VectorStoreProviderConfig {
    /// Provider name (e.g., "milvus", "memory", "null")
    pub provider: String,
    /// Connection URI or path
    pub uri: Option<String>,
    /// Collection/index name
    pub collection: Option<String>,
    /// Embedding dimensions
    pub dimensions: Option<usize>,
    /// API key or token for authentication
    pub api_key: Option<String>,
    /// Enable encryption
    pub encrypted: Option<bool>,
    /// Encryption key (if encrypted)
    pub encryption_key: Option<String>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(VectorStoreProviderConfig {
    /// Set the URI
    uri: with_uri(into String),
    /// Set the collection name
    collection: with_collection(into String),
    /// Set the embedding dimensions
    dimensions: with_dimensions(usize),
    /// Set the API key
    api_key: with_api_key(into String),
});

impl VectorStoreProviderConfig {
    /// Enable encryption
    #[must_use]
    pub fn with_encryption(mut self, key: impl Into<String>) -> Self {
        self.encrypted = Some(true);
        self.encryption_key = Some(key.into());
        self
    }
}

crate::impl_registry!(
    provider_trait: crate::ports::providers::VectorStoreProvider,
    config_type: VectorStoreProviderConfig,
    entry_type: VectorStoreProviderEntry,
    slice_name: VECTOR_STORE_PROVIDERS,
    resolve_fn: resolve_vector_store_provider,
    list_fn: list_vector_store_providers
);
