//! Factory implementations for creating providers

use crate::core::{error::{Error, Result}, types::{EmbeddingConfig, VectorStoreConfig}};
use crate::providers::{EmbeddingProvider, VectorStoreProvider, MockEmbeddingProvider, InMemoryVectorStoreProvider};
use async_trait::async_trait;
use std::sync::Arc;

/// Provider factory trait
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    async fn create_embedding_provider(&self, config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingProvider>>;
    async fn create_vector_store_provider(&self, config: &VectorStoreConfig) -> Result<Arc<dyn VectorStoreProvider>>;
    fn supported_embedding_providers(&self) -> Vec<String>;
    fn supported_vector_store_providers(&self) -> Vec<String>;
}

/// Default provider factory implementation
pub struct DefaultProviderFactory;

impl DefaultProviderFactory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderFactory for DefaultProviderFactory {
    async fn create_embedding_provider(&self, config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingProvider>> {
        match config.provider.as_str() {
            "mock" => Ok(Arc::new(MockEmbeddingProvider::new())),
            _ => Err(Error::generic(format!("Unsupported embedding provider: {}", config.provider))),
        }
    }

    async fn create_vector_store_provider(&self, config: &VectorStoreConfig) -> Result<Arc<dyn VectorStoreProvider>> {
        match config.provider.as_str() {
            "in-memory" => Ok(Arc::new(InMemoryVectorStoreProvider::new())),
            _ => Err(Error::generic(format!("Unsupported vector store provider: {}", config.provider))),
        }
    }

    fn supported_embedding_providers(&self) -> Vec<String> {
        vec!["mock".to_string()]
    }

    fn supported_vector_store_providers(&self) -> Vec<String> {
        vec!["in-memory".to_string()]
    }
}

impl Default for DefaultProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Service provider for dependency injection
pub struct ServiceProvider {
    factory: DefaultProviderFactory,
    registry: crate::registry::ProviderRegistry,
}

impl ServiceProvider {
    pub fn new() -> Self {
        Self {
            factory: DefaultProviderFactory::new(),
            registry: crate::registry::ProviderRegistry::new(),
        }
    }

    pub async fn get_embedding_provider(&self, config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingProvider>> {
        self.factory.create_embedding_provider(config).await
    }

    pub async fn get_vector_store_provider(&self, config: &VectorStoreConfig) -> Result<Arc<dyn VectorStoreProvider>> {
        self.factory.create_vector_store_provider(config).await
    }

    pub fn registry(&self) -> &crate::registry::ProviderRegistry {
        &self.registry
    }
}

impl Default for ServiceProvider {
    fn default() -> Self {
        Self::new()
    }
}