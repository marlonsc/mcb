//! OpenAI Embedding Provider
//!
//! Implements the EmbeddingProvider port using OpenAI's embedding API.
//! Supports text-embedding-3-small, text-embedding-3-large, and ada-002.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::constants::{
    EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_LARGE,
    EMBEDDING_DIMENSION_OPENAI_SMALL,
};
use crate::provider_utils::{JsonRequestParams, send_json_request};
use crate::utils::http::RequestErrorKind;
use crate::utils::parse_embedding_vector;
use mcb_domain::constants::http::CONTENT_TYPE_JSON;

use super::helpers::{HttpEmbeddingClient, process_batch};

/// OpenAI embedding provider
///
/// Implements the `EmbeddingProvider` domain port using OpenAI's embedding API.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::OpenAIEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = OpenAIEmbeddingProvider::new(
///         "sk-your-api-key".to_string(),
///         None,
///         "text-embedding-3-small".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
pub struct OpenAIEmbeddingProvider {
    client: HttpEmbeddingClient,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `base_url` - Optional custom base URL (defaults to OpenAI API)
    /// * `model` - Model name (e.g., "text-embedding-3-small")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
        Self {
            client: HttpEmbeddingClient::new(
                api_key,
                base_url,
                "https://api.openai.com/v1",
                model,
                timeout,
                http_client,
            ),
        }
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> &str {
        &self.client.base_url
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.client.model
    }

    /// Get the maximum tokens for this model
    pub fn max_tokens(&self) -> usize {
        match self.client.model.as_str() {
            "text-embedding-3-small" => 8192,
            "text-embedding-3-large" => 8192,
            "text-embedding-ada-002" => 8192,
            _ => 8192, // Default fallback
        }
    }

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "input": texts,
            "model": self.client.model,
            "encoding_format": "float"
        });

        let headers = vec![
            ("Authorization", format!("Bearer {}", self.client.api_key)),
            ("Content-Type", CONTENT_TYPE_JSON.to_string()),
        ];

        send_json_request(JsonRequestParams {
            client: &self.client.client,
            method: reqwest::Method::POST,
            url: format!("{}/embeddings", self.base_url()),
            timeout: self.client.timeout,
            provider: "OpenAI",
            operation: "embeddings",
            kind: RequestErrorKind::Embedding,
            headers: &headers,
            body: Some(&payload),
        })
        .await
    }

    /// Parse embedding vector from response data
    fn parse_embedding(&self, index: usize, item: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = parse_embedding_vector(item, "embedding", index)?;

        Ok(Embedding {
            vector: embedding_vec,
            model: self.client.model.clone(),
            dimensions: self.dimensions(),
        })
    }
}

#[async_trait]
/// OpenAI implementation of the EmbeddingProvider trait.
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    /// Generates embeddings for a batch of texts.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        process_batch(texts, self.fetch_embeddings(texts), |i, item| {
            self.parse_embedding(i, item)
        })
        .await
    }

    fn dimensions(&self) -> usize {
        match self.client.model.as_str() {
            "text-embedding-3-small" => EMBEDDING_DIMENSION_OPENAI_SMALL,
            "text-embedding-3-large" => EMBEDDING_DIMENSION_OPENAI_LARGE,
            "text-embedding-ada-002" => EMBEDDING_DIMENSION_OPENAI_ADA,
            _ => EMBEDDING_DIMENSION_OPENAI_SMALL,
        }
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;
use mcb_domain::registry::embedding::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};

/// Factory function for creating OpenAI embedding provider instances.
fn openai_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    use crate::utils::http::create_http_provider_config;

    let cfg = create_http_provider_config(config, "OpenAI", "text-embedding-3-small")?;

    Ok(Arc::new(OpenAIEmbeddingProvider::new(
        cfg.api_key,
        cfg.base_url,
        cfg.model,
        cfg.timeout,
        cfg.client,
    )))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OPENAI_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "openai",
    description: "OpenAI embedding provider (text-embedding-3-small/large, ada-002)",
    factory: openai_factory,
};
