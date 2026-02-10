//! VoyageAI Embedding Provider
//!
//! Implements the EmbeddingProvider port using VoyageAI's embedding API.
//! Optimized for code embeddings with voyage-code-3 model.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::constants::{
    CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_VOYAGEAI_CODE, EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
    VOYAGEAI_MAX_INPUT_TOKENS,
};
use crate::embedding::helpers::constructor;
use crate::utils::HttpResponseUtils;
use crate::utils::http::{
    create_http_provider_config, handle_request_error, parse_embedding_vector,
};

/// VoyageAI embedding provider
///
/// Implements the `EmbeddingProvider` domain port using VoyageAI's embedding API.
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
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Client,
}

impl VoyageAIEmbeddingProvider {
    /// Create a new VoyageAI embedding provider
    ///
    /// # Arguments
    /// * `api_key` - VoyageAI API key
    /// * `base_url` - Optional custom base URL (defaults to VoyageAI API)
    /// * `model` - Model name (e.g., "voyage-code-3")
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

    /// Get the effective base URL
    fn effective_base_url(&self) -> String {
        constructor::get_effective_url(self.base_url.as_deref(), "https://api.voyageai.com/v1")
    }

    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        // All VoyageAI models support the same max tokens
        VOYAGEAI_MAX_INPUT_TOKENS
    }

    /// Get the API key for this provider
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> String {
        self.effective_base_url()
    }

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model
        });

        let response = self
            .http_client
            .post(format!("{}/embeddings", self.effective_base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", CONTENT_TYPE_JSON)
            .timeout(self.timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|e| handle_request_error(e, self.timeout, "VoyageAI"))?;

        HttpResponseUtils::check_and_parse(response, "VoyageAI").await
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
impl EmbeddingProvider for VoyageAIEmbeddingProvider {
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
            "voyage-code-3" => EMBEDDING_DIMENSION_VOYAGEAI_CODE,
            _ => EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
        }
    }

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

/// Factory function for creating VoyageAI embedding provider instances.
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
