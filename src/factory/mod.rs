//! Factory implementations for creating providers

use crate::core::{
    error::{Error, Result},
    types::{EmbeddingConfig, VectorStoreConfig},
};
use crate::providers::{
    embedding::MockEmbeddingProvider,
    EmbeddingProvider, VectorStoreProvider,
};

// Import individual providers that exist
use crate::providers::embedding::ollama::OllamaEmbeddingProvider;
use crate::providers::embedding::openai::OpenAIEmbeddingProvider;
use crate::providers::embedding::voyageai::VoyageAIEmbeddingProvider;
use crate::providers::vector_store::milvus::MilvusVectorStoreProvider;
use crate::providers::vector_store::InMemoryVectorStoreProvider;
use async_trait::async_trait;
use std::sync::Arc;

/// Provider factory trait
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    async fn create_embedding_provider(
        &self,
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>>;
    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>>;
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
    async fn create_embedding_provider(
        &self,
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        match config.provider.as_str() {
            "openai" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("OpenAI API key required"))?;
                Ok(Arc::new(OpenAIEmbeddingProvider::new(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                )))
            }
            "ollama" => Ok(Arc::new(OllamaEmbeddingProvider::new(
                config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string()),
                config.model.clone(),
            ))),
            "voyageai" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("VoyageAI API key required"))?;
                Ok(Arc::new(VoyageAIEmbeddingProvider::new(
                    api_key.clone(),
                    config.model.clone(),
                )))
            }
            "mock" => Ok(Arc::new(MockEmbeddingProvider::new())),
            _ => Err(Error::config(format!(
                "Unsupported embedding provider: {}",
                config.provider
            ))),
        }
    }

    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        match config.provider.as_str() {
            "in-memory" => Ok(Arc::new(InMemoryVectorStoreProvider::new())),
            "milvus" => {
                let address = config
                    .address
                    .as_ref()
                    .ok_or_else(|| Error::config("Milvus address required"))?;
                Ok(Arc::new(
                    MilvusVectorStoreProvider::new(address.clone(), config.token.clone()).await?,
                ))
            }
            _ => Err(Error::config(format!(
                "Unsupported vector store provider: {}",
                config.provider
            ))),
        }
    }

    fn supported_embedding_providers(&self) -> Vec<String> {
        vec![
            "openai".to_string(),
            "ollama".to_string(),
            "voyageai".to_string(),
            "mock".to_string(),
        ]
    }

    fn supported_vector_store_providers(&self) -> Vec<String> {
        vec!["in-memory".to_string(), "milvus".to_string()]
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

    pub async fn get_embedding_provider(
        &self,
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        self.factory.create_embedding_provider(config).await
    }

    pub async fn get_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
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
