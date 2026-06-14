//! Provider admin interfaces (embedding, vector store, language).

use serde::{Deserialize, Serialize};

use crate::registry::embedding::EmbeddingProviderConfig;
use crate::registry::language::LanguageProviderConfig;
use crate::registry::vector_store::VectorStoreProviderConfig;

/// Information about an available provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Unique name identifier for the provider
    pub name: String,
    /// Human-readable description of the provider
    pub description: String,
}

impl ProviderInfo {
    /// Create a new `ProviderInfo` instance
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

crate::provider_admin_interface!(
    /// Interface for embedding provider admin operations.
    trait EmbeddingAdminInterface,
    config = EmbeddingProviderConfig,
    list_doc = "List all available embedding providers.",
    extra = {
        /// Get current provider name.
        fn current_provider(&self) -> String;
    }
);

crate::provider_admin_interface!(
    /// Interface for vector store provider admin operations.
    trait VectorStoreAdminInterface,
    config = VectorStoreProviderConfig,
    list_doc = "List all available vector store providers.",
    extra = {}
);

crate::provider_admin_interface!(
    /// Interface for language provider admin operations.
    trait LanguageAdminInterface,
    config = LanguageProviderConfig,
    list_doc = "List all available language providers.",
    extra = {}
);
