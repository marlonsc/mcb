//! Provider registry for dependency injection

use crate::core::error::{Error, Result};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe provider registry
#[derive(Clone)]
pub struct ProviderRegistry {
    embedding_providers: Arc<RwLock<HashMap<String, Arc<dyn EmbeddingProvider>>>>,
    vector_store_providers: Arc<RwLock<HashMap<String, Arc<dyn VectorStoreProvider>>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            embedding_providers: Arc::new(RwLock::new(HashMap::new())),
            vector_store_providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an embedding provider
    pub fn register_embedding_provider(
        &self,
        name: impl Into<String>,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        let name = name.into();
        let mut providers = self.embedding_providers.write().unwrap();

        if providers.contains_key(&name) {
            return Err(Error::generic(format!(
                "Embedding provider '{}' already registered",
                name
            )));
        }

        providers.insert(name, provider);
        Ok(())
    }

    /// Register a vector store provider
    pub fn register_vector_store_provider(
        &self,
        name: impl Into<String>,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        let name = name.into();
        let mut providers = self.vector_store_providers.write().unwrap();

        if providers.contains_key(&name) {
            return Err(Error::generic(format!(
                "Vector store provider '{}' already registered",
                name
            )));
        }

        providers.insert(name, provider);
        Ok(())
    }

    /// Get an embedding provider by name
    pub fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>> {
        let providers = self.embedding_providers.read().unwrap();
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("Embedding provider '{}' not found", name)))
    }

    /// Get a vector store provider by name
    pub fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>> {
        let providers = self.vector_store_providers.read().unwrap();
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("Vector store provider '{}' not found", name)))
    }

    /// List all registered embedding providers
    pub fn list_embedding_providers(&self) -> Vec<String> {
        let providers = self.embedding_providers.read().unwrap();
        providers.keys().cloned().collect()
    }

    /// List all registered vector store providers
    pub fn list_vector_store_providers(&self) -> Vec<String> {
        let providers = self.vector_store_providers.read().unwrap();
        providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
