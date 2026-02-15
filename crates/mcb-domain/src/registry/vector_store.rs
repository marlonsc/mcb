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

impl VectorStoreProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Enable encryption
    pub fn with_encryption(mut self, key: impl Into<String>) -> Self {
        self.encrypted = Some(true);
        self.encryption_key = Some(key.into());
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    /// Set the URI
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the collection name
    pub fn with_collection(mut self, collection: impl Into<String>) -> Self {
        self.collection = Some(collection.into());
        self
    }

    /// Set the dimensions
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
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
