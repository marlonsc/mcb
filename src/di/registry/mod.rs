//! Provider registry for dependency injection

use crate::core::error::{Error, Result};
use crate::core::locks::{lock_rwlock_read, lock_rwlock_write};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for provider registry
pub trait ProviderRegistryTrait: Send + Sync {
    fn register_embedding_provider(
        &self,
        name: String,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()>;

    fn register_vector_store_provider(
        &self,
        name: String,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()>;

    fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>>;
    fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>>;
    fn list_embedding_providers(&self) -> Vec<String>;
    fn list_vector_store_providers(&self) -> Vec<String>;
}

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
}

impl ProviderRegistryTrait for ProviderRegistry {
    /// Register an embedding provider
    fn register_embedding_provider(
        &self,
        name: String,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        let mut providers = lock_rwlock_write(
            &self.embedding_providers,
            "ProviderRegistry::register_embedding_provider",
        )?;

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
    fn register_vector_store_provider(
        &self,
        name: String,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        let mut providers = lock_rwlock_write(
            &self.vector_store_providers,
            "ProviderRegistry::register_vector_store_provider",
        )?;

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
    fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>> {
        let providers = lock_rwlock_read(
            &self.embedding_providers,
            "ProviderRegistry::get_embedding_provider",
        )?;
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("Embedding provider '{}' not found", name)))
    }

    /// Get a vector store provider by name
    fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>> {
        let providers = lock_rwlock_read(
            &self.vector_store_providers,
            "ProviderRegistry::get_vector_store_provider",
        )?;
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("Vector store provider '{}' not found", name)))
    }

    /// List all registered embedding providers
    fn list_embedding_providers(&self) -> Vec<String> {
        let providers = match lock_rwlock_read(
            &self.embedding_providers,
            "ProviderRegistry::list_embedding_providers",
        ) {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        providers.keys().cloned().collect()
    }

    /// List all registered vector store providers
    fn list_vector_store_providers(&self) -> Vec<String> {
        let providers = match lock_rwlock_read(
            &self.vector_store_providers,
            "ProviderRegistry::list_vector_store_providers",
        ) {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
