//! Factory implementations for creating providers

use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::constants::HTTP_REQUEST_TIMEOUT;
use crate::infrastructure::di::factory::traits::{ProviderFactory, ServiceProviderInterface};
use crate::infrastructure::di::registry::ProviderRegistry;
use crate::infrastructure::di::registry::ProviderRegistryTrait;
use crate::{EmbeddingConfig, Error, Result, VectorStoreConfig};

/// Type-safe enum for embedding provider types
///
/// This enum replaces string-based provider selection with compile-time type safety.
/// Prevents typos and provides exhaustive pattern matching in the factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmbeddingProviderType {
    OpenAI,
    Ollama,
    VoyageAI,
    Gemini,
    FastEmbed,
}

impl EmbeddingProviderType {
    /// Parse a provider string into the enum variant
    ///
    /// Returns an error if the provider name is not recognized.
    fn from_config_string(provider: &str) -> Result<Self> {
        match provider.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "ollama" => Ok(Self::Ollama),
            "voyageai" => Ok(Self::VoyageAI),
            "gemini" => Ok(Self::Gemini),
            "fastembed" => Ok(Self::FastEmbed),
            _ => Err(Error::config(format!(
                "Unsupported embedding provider: {}. \
                 Supported providers: openai, ollama, voyageai, gemini, fastembed",
                provider
            ))),
        }
    }
}

// Import individual providers that exist
use crate::adapters::providers::embedding::fastembed::FastEmbedProvider;
use crate::adapters::providers::embedding::gemini::GeminiEmbeddingProvider;
// NullEmbeddingProvider removed from production code paths - use a real provider
use crate::adapters::providers::embedding::ollama::OllamaEmbeddingProvider;
use crate::adapters::providers::embedding::openai::OpenAIEmbeddingProvider;
use crate::adapters::providers::embedding::voyageai::VoyageAIEmbeddingProvider;
#[cfg(feature = "milvus")]
use crate::adapters::providers::vector_store::milvus::MilvusVectorStoreProvider;
use crate::adapters::providers::vector_store::InMemoryVectorStoreProvider;
use async_trait::async_trait;
use shaku::Component;
use std::sync::Arc;

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
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        // Convert string provider to type-safe enum
        let provider_type = EmbeddingProviderType::from_config_string(&config.provider)?;

        // Use exhaustive pattern matching on the enum
        match provider_type {
            EmbeddingProviderType::OpenAI => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("OpenAI API key required"))?;
                Ok(Arc::new(OpenAIEmbeddingProvider::new(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    HTTP_REQUEST_TIMEOUT,
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            EmbeddingProviderType::Ollama => {
                Ok(Arc::new(OllamaEmbeddingProvider::new(
                    config.base_url.clone().unwrap_or_else(|| {
                        crate::infrastructure::constants::OLLAMA_DEFAULT_URL.to_string()
                    }),
                    config.model.clone(),
                    HTTP_REQUEST_TIMEOUT,
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            EmbeddingProviderType::VoyageAI => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("VoyageAI API key required"))?;
                Ok(Arc::new(VoyageAIEmbeddingProvider::new(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            EmbeddingProviderType::Gemini => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("Gemini API key required"))?;
                Ok(Arc::new(GeminiEmbeddingProvider::new(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    HTTP_REQUEST_TIMEOUT,
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            EmbeddingProviderType::FastEmbed => {
                Ok(Arc::new(FastEmbedProvider::new()?) as Arc<dyn EmbeddingProvider>)
            }
        }
    }

    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        match config.provider.to_lowercase().as_str() {
            "in-memory" => {
                Ok(Arc::new(InMemoryVectorStoreProvider::new()) as Arc<dyn VectorStoreProvider>)
            }
            "filesystem" => {
                use crate::adapters::providers::vector_store::filesystem::{
                    FilesystemVectorStore, FilesystemVectorStoreConfig,
                };
                let base_path = config
                    .address
                    .as_ref()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("./data/vectors"));
                let fs_config = FilesystemVectorStoreConfig {
                    base_path,
                    dimensions: config.dimensions.unwrap_or(1536),
                    ..Default::default()
                };
                Ok(Arc::new(FilesystemVectorStore::new(fs_config).await?)
                    as Arc<dyn VectorStoreProvider>)
            }
            "milvus" => {
                #[cfg(feature = "milvus")]
                {
                    let address = config
                        .address
                        .as_ref()
                        .ok_or_else(|| Error::config("Milvus address required"))?;
                    Ok(Arc::new(
                        MilvusVectorStoreProvider::new(
                            address.clone(),
                            config.token.clone(),
                            config.timeout_secs,
                        )
                        .await?,
                    ) as Arc<dyn VectorStoreProvider>)
                }
                #[cfg(not(feature = "milvus"))]
                {
                    Err(Error::config("Milvus provider requested but the 'milvus' feature is not enabled. Recompile with --features milvus."))
                }
            }
            "edgevec" => {
                use crate::adapters::providers::vector_store::edgevec::{
                    EdgeVecConfig, EdgeVecVectorStoreProvider,
                };
                let edgevec_config = EdgeVecConfig {
                    dimensions: config.dimensions.unwrap_or(1536),
                    ..Default::default()
                };
                Ok(Arc::new(EdgeVecVectorStoreProvider::new(edgevec_config)?)
                    as Arc<dyn VectorStoreProvider>)
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
            "gemini".to_string(),
            "fastembed".to_string(),
        ]
    }

    fn supported_vector_store_providers(&self) -> Vec<String> {
        let mut providers = vec![
            "in-memory".to_string(),
            "filesystem".to_string(),
            "edgevec".to_string(),
        ];
        if cfg!(feature = "milvus") {
            providers.push("milvus".to_string());
        }
        providers
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
