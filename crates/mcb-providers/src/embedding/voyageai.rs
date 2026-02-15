//! `VoyageAI` Embedding Provider
//!
//! Implements the `EmbeddingProvider` port using `VoyageAI`'s embedding API.
//! Optimized for code embeddings with voyage-code-3 model.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::constants::embedding::{
    EMBEDDING_DIMENSION_VOYAGEAI_CODE, EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use super::helpers::{HttpEmbeddingClient, process_batch};
use crate::constants::VOYAGEAI_MAX_INPUT_TOKENS;
use crate::provider_utils::{JsonRequestParams, send_json_request};
use crate::utils::http::{RequestErrorKind, create_http_provider_config, parse_embedding_vector};
use mcb_domain::constants::http::CONTENT_TYPE_JSON;

/// `VoyageAI` embedding provider
///
/// Implements the `EmbeddingProvider` domain port using `VoyageAI`'s embedding API.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::VoyageAIEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() {
///     let client = Client::new();
///     let provider = VoyageAIEmbeddingProvider::new(
///         "voyage-your-api-key".to_string(),
///         None,
///         "voyage-code-3".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
/// }
/// ```
pub struct VoyageAIEmbeddingProvider {
    client: HttpEmbeddingClient,
}

impl VoyageAIEmbeddingProvider {
    /// Create a new `VoyageAI` embedding provider
    ///
    /// # Arguments
    /// * `api_key` - `VoyageAI` API key
    /// * `base_url` - Optional custom base URL (defaults to `VoyageAI` API)
    /// * `model` - Model name (e.g., "voyage-code-3")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    #[must_use]
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
                "https://api.voyageai.com/v1",
                model,
                timeout,
                http_client,
            ),
        }
    }

    /// Get the model name for this provider
    #[must_use]
    pub fn model(&self) -> &str {
        &self.client.model
    }

    /// Get the maximum tokens supported by this provider
    #[must_use]
    pub fn max_tokens(&self) -> usize {
        // All VoyageAI models support the same max tokens
        VOYAGEAI_MAX_INPUT_TOKENS
    }

    /// Get the API key for this provider
    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.client.api_key
    }

    /// Get the base URL for this provider
    #[must_use]
    pub fn base_url(&self) -> String {
        self.client.base_url.clone()
    }

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "input": texts,
            "model": self.client.model
        });

        let headers = vec![
            ("Authorization", format!("Bearer {}", self.client.api_key)),
            ("Content-Type", CONTENT_TYPE_JSON.to_owned()),
        ];

        send_json_request(JsonRequestParams {
            client: &self.client.client,
            method: reqwest::Method::POST,
            url: format!("{}/embeddings", self.client.base_url),
            timeout: self.client.timeout,
            provider: "VoyageAI",
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
/// `VoyageAI` implementation of the `EmbeddingProvider` trait.
impl EmbeddingProvider for VoyageAIEmbeddingProvider {
    /// Generates embeddings for a batch of texts.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        process_batch(texts, self.fetch_embeddings(texts), |i, item| {
            self.parse_embedding(i, item)
        })
        .await
    }

    /// Returns the embedding dimensions for the configured model.
    fn dimensions(&self) -> usize {
        match self.client.model.as_str() {
            "voyage-code-3" => EMBEDDING_DIMENSION_VOYAGEAI_CODE,
            _ => EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
        }
    }

    /// Returns the provider name ("voyageai").
    fn provider_name(&self) -> &str {
        "voyageai"
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

/// Factory function for creating `VoyageAI` embedding provider instances.
fn voyageai_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    let cfg = create_http_provider_config(config, "VoyageAI", "voyage-code-3")?;

    Ok(Arc::new(VoyageAIEmbeddingProvider::new(
        cfg.api_key,
        cfg.base_url,
        cfg.model,
        cfg.timeout,
        cfg.client,
    )))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static VOYAGEAI_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "voyageai",
    description: "VoyageAI embedding provider (voyage-code-3, etc.)",
    factory: voyageai_factory,
};
