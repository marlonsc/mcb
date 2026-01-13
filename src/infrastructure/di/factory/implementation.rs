//! Factory implementations for creating providers
//!
//! This module provides the factory pattern for creating embedding and vector store
//! providers. The actual provider creation logic is centralized in the dispatch module.

use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::domain::types::{EmbeddingProviderKind, VectorStoreProviderKind};
use crate::infrastructure::di::dispatch::{
    create_embedding_provider_from_config, create_vector_store_provider_from_config,
};
use crate::infrastructure::di::factory::traits::{ProviderFactory, ServiceProviderInterface};
use crate::infrastructure::di::registry::ProviderRegistry;
use crate::infrastructure::di::registry::ProviderRegistryTrait;
use crate::{EmbeddingConfig, Result, VectorStoreConfig};
use async_trait::async_trait;
use shaku::Component;
use std::sync::Arc;

/// Default provider factory implementation
///
/// Delegates to the centralized dispatch module for provider creation.
/// This ensures single source of truth for all provider instantiation logic.
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
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        // Delegate to centralized dispatch
        create_embedding_provider_from_config(config, http_client).await
    }

    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        // Delegate to centralized dispatch
        create_vector_store_provider_from_config(config).await
    }

    fn supported_embedding_providers(&self) -> Vec<String> {
        EmbeddingProviderKind::supported_providers()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn supported_vector_store_providers(&self) -> Vec<String> {
        VectorStoreProviderKind::supported_providers()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for DefaultProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Service provider for dependency injection
#[derive(Component)]
#[shaku(interface = ServiceProviderInterface)]
pub struct ServiceProvider {
    #[shaku(default = DefaultProviderFactory::new())]
    factory: DefaultProviderFactory,
    #[shaku(default = ProviderRegistry::new())]
    registry: ProviderRegistry,
}

#[async_trait]
impl ServiceProviderInterface for ServiceProvider {
    fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    fn list_providers(&self) -> (Vec<String>, Vec<String>) {
        (
            self.registry.list_embedding_providers(),
            self.registry.list_vector_store_providers(),
        )
    }

    fn register_embedding_provider(
        &self,
        name: &str,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        self.registry
            .register_embedding_provider(name.to_string(), provider)
    }

    fn register_vector_store_provider(
        &self,
        name: &str,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        self.registry
            .register_vector_store_provider(name.to_string(), provider)
    }

    fn remove_embedding_provider(&self, name: &str) -> Result<()> {
        self.registry.remove_embedding_provider(name)
    }

    fn remove_vector_store_provider(&self, name: &str) -> Result<()> {
        self.registry.remove_vector_store_provider(name)
    }

    async fn get_embedding_provider(
        &self,
        config: &EmbeddingConfig,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        // First try to get from registry
        if let Ok(provider) = self.registry.get_embedding_provider(&config.provider) {
            return Ok(provider);
        }

        // If not found, create via factory and register
        let provider = self
            .factory
            .create_embedding_provider(config, http_client)
            .await?;
        self.registry
            .register_embedding_provider(config.provider.clone(), Arc::clone(&provider))?;

        Ok(provider)
    }

    async fn get_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        // DIAGNOSTIC: Log provider creation for debugging
        tracing::debug!("=== Factory Creating Vector Store ===");
        tracing::debug!("Provider requested: {}", config.provider);
        tracing::debug!("Address: {:?}", config.address);
        tracing::debug!("Collection: {:?}", config.collection);
        tracing::debug!("====================================");

        // First try to get from registry
        if let Ok(provider) = self.registry.get_vector_store_provider(&config.provider) {
            return Ok(provider);
        }

        // If not found, create via factory and register
        let provider = self.factory.create_vector_store_provider(config).await?;
        self.registry
            .register_vector_store_provider(config.provider.clone(), Arc::clone(&provider))?;

        Ok(provider)
    }
}

impl ServiceProvider {
    pub fn new() -> Self {
        Self {
            factory: DefaultProviderFactory::new(),
            registry: ProviderRegistry::new(),
        }
    }
}

impl Default for ServiceProvider {
    fn default() -> Self {
        Self::new()
    }
}
