//! Provider Factory
//!
//! Factory for creating embedding and vector store providers based on configuration.
//! Follows the Factory pattern to abstract provider instantiation.
//! Uses DI container for dependency resolution instead of direct instantiation.

use std::sync::Arc;
use std::time::Duration;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};

use crate::adapters::http_client::{HttpClientPool, HttpClientProvider};
use crate::adapters::providers::embedding::{
    FastEmbedProvider, GeminiEmbeddingProvider, NullEmbeddingProvider, OllamaEmbeddingProvider,
    OpenAIEmbeddingProvider, VoyageAIEmbeddingProvider,
};
use crate::adapters::providers::vector_store::{
    EncryptedVectorStoreProvider, InMemoryVectorStoreProvider, NullVectorStoreProvider,
};
use crate::constants::OLLAMA_DEFAULT_PORT;
use crate::crypto::CryptoService;

/// Known embedding provider names
pub mod embedding_providers {
    pub const OPENAI: &str = "openai";
    pub const VOYAGEAI: &str = "voyageai";
    pub const OLLAMA: &str = "ollama";
    pub const GEMINI: &str = "gemini";
    pub const FASTEMBED: &str = "fastembed";
    pub const NULL: &str = "null";
}

/// Known vector store provider names
pub mod vector_store_providers {
    pub const IN_MEMORY: &str = "in_memory";
    pub const MEMORY: &str = "memory";
    pub const ENCRYPTED: &str = "encrypted";
    pub const FILESYSTEM: &str = "filesystem";
    pub const NULL: &str = "null";
}

/// Factory for creating embedding providers
pub struct EmbeddingProviderFactory;

impl EmbeddingProviderFactory {
    /// Create an embedding provider based on configuration
    ///
    /// # Arguments
    /// * `config` - Embedding provider configuration
    /// * `http_client` - Optional HTTP client for API-based providers
    ///
    /// # Returns
    /// * `Result<Arc<dyn EmbeddingProvider>>` - The configured provider
    pub fn create(
        config: &EmbeddingConfig,
        http_client: Option<Arc<dyn HttpClientProvider>>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let provider_name = config.provider.to_lowercase();
        let timeout = Duration::from_secs(30);

        match provider_name.as_str() {
            embedding_providers::NULL => Ok(Arc::new(NullEmbeddingProvider::new())),

            embedding_providers::OPENAI => {
                let http_client = http_client.or_else(|| Self::create_default_http_client().ok());
                let http_client = http_client.ok_or_else(|| Error::Configuration {
                    message: "HTTP client required for OpenAI provider".to_string(),
                    source: None,
                })?;

                let api_key = config.api_key.clone().ok_or_else(|| Error::Configuration {
                    message: "API key required for OpenAI provider".to_string(),
                    source: None,
                })?;

                Ok(Arc::new(OpenAIEmbeddingProvider::new(
                    api_key,
                    config.base_url.clone(),
                    config.model.clone(),
                    timeout,
                    http_client,
                )))
            }

            embedding_providers::VOYAGEAI => {
                let http_client = http_client.or_else(|| Self::create_default_http_client().ok());
                let http_client = http_client.ok_or_else(|| Error::Configuration {
                    message: "HTTP client required for VoyageAI provider".to_string(),
                    source: None,
                })?;

                let api_key = config.api_key.clone().ok_or_else(|| Error::Configuration {
                    message: "API key required for VoyageAI provider".to_string(),
                    source: None,
                })?;

                Ok(Arc::new(VoyageAIEmbeddingProvider::new(
                    api_key,
                    config.base_url.clone(),
                    config.model.clone(),
                    http_client,
                )))
            }

            embedding_providers::OLLAMA => {
                let http_client = http_client.or_else(|| Self::create_default_http_client().ok());
                let http_client = http_client.ok_or_else(|| Error::Configuration {
                    message: "HTTP client required for Ollama provider".to_string(),
                    source: None,
                })?;

                // Ollama defaults to localhost with standard port
                let base_url = config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| format!("http://localhost:{}", OLLAMA_DEFAULT_PORT));

                Ok(Arc::new(OllamaEmbeddingProvider::new(
                    base_url,
                    config.model.clone(),
                    timeout,
                    http_client,
                )))
            }

            embedding_providers::GEMINI => {
                let http_client = http_client.or_else(|| Self::create_default_http_client().ok());
                let http_client = http_client.ok_or_else(|| Error::Configuration {
                    message: "HTTP client required for Gemini provider".to_string(),
                    source: None,
                })?;

                let api_key = config.api_key.clone().ok_or_else(|| Error::Configuration {
                    message: "API key required for Gemini provider".to_string(),
                    source: None,
                })?;

                Ok(Arc::new(GeminiEmbeddingProvider::new(
                    api_key,
                    config.base_url.clone(),
                    config.model.clone(),
                    timeout,
                    http_client,
                )))
            }

            embedding_providers::FASTEMBED => {
                let provider = FastEmbedProvider::new()?;
                Ok(Arc::new(provider))
            }

            _ => Err(Error::Configuration {
                message: format!("Unknown embedding provider: {}", config.provider),
                source: None,
            }),
        }
    }

    /// Create a default null provider (for testing/development)
    pub fn create_null() -> Arc<dyn EmbeddingProvider> {
        Arc::new(NullEmbeddingProvider::new())
    }

    /// Create default HTTP client for providers that need it
    fn create_default_http_client() -> Result<Arc<dyn HttpClientProvider>> {
        HttpClientPool::new()
            .map(|pool| Arc::new(pool) as Arc<dyn HttpClientProvider>)
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to create HTTP client: {}", e),
                source: None,
            })
    }
}

/// Factory for creating vector store providers
pub struct VectorStoreProviderFactory;

impl VectorStoreProviderFactory {
    /// Create a vector store provider based on configuration
    ///
    /// # Arguments
    /// * `config` - Vector store provider configuration
    /// * `crypto_service` - Optional crypto service for encrypted providers
    ///
    /// # Returns
    /// * `Result<Arc<dyn VectorStoreProvider>>` - The configured provider
    pub fn create(
        config: &VectorStoreConfig,
        crypto_service: Option<&CryptoService>,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        let provider_name = config.provider.to_lowercase();

        match provider_name.as_str() {
            vector_store_providers::NULL => Ok(Arc::new(NullVectorStoreProvider::new())),

            vector_store_providers::IN_MEMORY | vector_store_providers::MEMORY => {
                Ok(Arc::new(InMemoryVectorStoreProvider::new()))
            }

            vector_store_providers::ENCRYPTED => {
                let crypto = crypto_service.ok_or_else(|| Error::Configuration {
                    message: "CryptoService required for encrypted vector store".to_string(),
                    source: None,
                })?;

                // Use in-memory as the underlying store for encrypted
                let inner = InMemoryVectorStoreProvider::new();
                Ok(Arc::new(EncryptedVectorStoreProvider::with_crypto_service(
                    inner,
                    Arc::new(crypto.clone()),
                )))
            }

            vector_store_providers::FILESYSTEM => {
                // NOTE: Filesystem provider uses in-memory storage as placeholder
                // Actual filesystem-backed vector store planned for future release
                Ok(Arc::new(InMemoryVectorStoreProvider::new()))
            }

            _ => Err(Error::Configuration {
                message: format!("Unknown vector store provider: {}", config.provider),
                source: None,
            }),
        }
    }

    /// Create a default in-memory provider
    pub fn create_in_memory() -> Arc<dyn VectorStoreProvider> {
        Arc::new(InMemoryVectorStoreProvider::new())
    }

    /// Create a null provider (for testing)
    pub fn create_null() -> Arc<dyn VectorStoreProvider> {
        Arc::new(NullVectorStoreProvider::new())
    }
}
