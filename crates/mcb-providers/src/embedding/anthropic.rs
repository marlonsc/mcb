//! Anthropic Embedding Provider
//!
//! Implements the EmbeddingProvider port using Anthropic's Voyage embedding API.
//! Anthropic partners with Voyage AI for embeddings, accessible via the Anthropic API.
//! Supports voyage-3, voyage-3-lite, and voyage-code-3 models.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::constants::{
    ANTHROPIC_MAX_INPUT_TOKENS, CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_ANTHROPIC_CODE,
    EMBEDDING_DIMENSION_ANTHROPIC_DEFAULT, EMBEDDING_DIMENSION_ANTHROPIC_LITE,
};
use crate::embedding::helpers::constructor;
use crate::utils::{HttpResponseUtils, handle_request_error, parse_embedding_vector};

/// Anthropic embedding provider
///
/// Implements the `EmbeddingProvider` domain port using Anthropic's embedding API.
/// Uses Voyage AI models accessed through the Anthropic API endpoint.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::AnthropicEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = AnthropicEmbeddingProvider::new(
///         "your-api-key".to_string(),
///         None,
///         "voyage-3".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
pub struct AnthropicEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Client,
}

impl AnthropicEmbeddingProvider {
    /// Create a new Anthropic embedding provider
    ///
    /// # Arguments
    /// * `api_key` - Anthropic API key
    /// * `base_url` - Optional custom base URL (defaults to Anthropic API)
    /// * `model` - Model name (e.g., "voyage-3", "voyage-code-3")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        timeout: Duration,
        http_client: Client,
    ) -> Self {
        let api_key = constructor::validate_api_key(&api_key);
        let base_url = constructor::validate_url(base_url);

        Self {
            api_key,
            base_url,
            model,
            timeout,
            http_client,
        }
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.voyageai.com/v1")
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens for this model
    pub fn max_tokens(&self) -> usize {
        // All Voyage AI models support the same max tokens
        ANTHROPIC_MAX_INPUT_TOKENS
    }

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model,
            "input_type": "document",
            "encoding_format": "float"
        });

        let response = self
            .http_client
            .post(format!("{}/embeddings", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", CONTENT_TYPE_JSON)
            .timeout(self.timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|e| handle_request_error(e, self.timeout, "Anthropic"))?;

        HttpResponseUtils::check_and_parse(response, "Anthropic").await
    }

    /// Parse embedding vector from response data
    fn parse_embedding(&self, index: usize, item: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = parse_embedding_vector(item, "embedding", index)?;

        Ok(Embedding {
            vector: embedding_vec,
            model: self.model.clone(),
            dimensions: self.dimensions(),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for AnthropicEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let response_data = self.fetch_embeddings(texts).await?;

        let data = response_data["data"].as_array().ok_or_else(|| {
            Error::embedding("Invalid response format: missing data array".to_string())
        })?;

        if data.len() != texts.len() {
            return Err(Error::embedding(format!(
                "Response data count mismatch: expected {}, got {}",
                texts.len(),
                data.len()
            )));
        }

        data.iter()
            .enumerate()
            .map(|(i, item)| self.parse_embedding(i, item))
            .collect()
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "voyage-3" => EMBEDDING_DIMENSION_ANTHROPIC_DEFAULT,
            "voyage-3-lite" => EMBEDDING_DIMENSION_ANTHROPIC_LITE,
            "voyage-code-3" => EMBEDDING_DIMENSION_ANTHROPIC_CODE,
            _ => EMBEDDING_DIMENSION_ANTHROPIC_DEFAULT,
        }
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;
use mcb_domain::registry::{EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry};

/// Factory function for creating Anthropic embedding provider instances.
fn anthropic_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    use crate::utils::http::create_http_provider_config;

    let cfg = create_http_provider_config(config, "Anthropic", "voyage-3")?;

    Ok(Arc::new(AnthropicEmbeddingProvider::new(
        cfg.api_key,
        cfg.base_url,
        cfg.model,
        cfg.timeout,
        cfg.client,
    )))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static ANTHROPIC_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "anthropic",
    description: "Anthropic embedding provider (voyage-3, voyage-3-lite, voyage-code-3)",
    factory: anthropic_factory,
};
