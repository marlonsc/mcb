//! Provider configuration manager ports.

use crate::error::Result;
use crate::value_objects::{EmbeddingConfig, VectorStoreConfig};

/// Provider configuration manager interface
#[async_trait::async_trait]
pub trait ProviderConfigManagerInterface: Send + Sync {
    /// Get embedding configuration by provider name.
    ///
    /// # Errors
    /// Returns an error if the named provider is not configured.
    fn get_embedding_config(&self, name: &str) -> Result<&EmbeddingConfig>;

    /// Get vector store configuration by provider name.
    ///
    /// # Errors
    /// Returns an error if the named provider is not configured.
    fn get_vector_store_config(&self, name: &str) -> Result<&VectorStoreConfig>;
    /// List all available embedding provider implementation names.
    fn list_embedding_providers(&self) -> Vec<String>;
    /// List all available vector store provider implementation names.
    fn list_vector_store_providers(&self) -> Vec<String>;

    /// Check if a specific embedding provider is available.
    fn has_embedding_provider(&self, name: &str) -> bool {
        self.list_embedding_providers().contains(&name.to_owned())
    }

    /// Check if a specific vector store provider is available.
    fn has_vector_store_provider(&self, name: &str) -> bool {
        self.list_vector_store_providers()
            .contains(&name.to_owned())
    }

    /// Get the configuration for the default embedding provider.
    fn get_default_embedding_config(&self) -> Option<&EmbeddingConfig> {
        let providers = self.list_embedding_providers();
        if providers.is_empty() {
            None
        } else {
            self.get_embedding_config(&providers[0]).ok()
        }
    }

    /// Get the configuration for the default vector store provider.
    fn get_default_vector_store_config(&self) -> Option<&VectorStoreConfig> {
        let providers = self.list_vector_store_providers();
        if providers.is_empty() {
            None
        } else {
            self.get_vector_store_config(&providers[0]).ok()
        }
    }
}
