//! Gemini Embedding Provider
//!
//! Implements the EmbeddingProvider port using Google's Gemini embedding API.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::constants::{CONTENT_TYPE_JSON, EMBEDDING_DIMENSION_GEMINI};
use crate::embedding::helpers::constructor;
use crate::provider_utils::{JsonRequestParams, parse_float_array_lossy, send_json_request};
use crate::utils::http::RequestErrorKind;

/// Gemini embedding provider
///
/// Implements the `EmbeddingProvider` domain port using Google's Gemini embedding API.
/// Receives HTTP client via constructor injection.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_providers::embedding::GeminiEmbeddingProvider;
/// use reqwest::Client;
/// use std::time::Duration;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::builder()
///         .timeout(Duration::from_secs(30))
///         .build()?;
///     let provider = GeminiEmbeddingProvider::new(
///         "AIza-your-api-key".to_string(),
///         None,
///         "text-embedding-004".to_string(),
///         Duration::from_secs(30),
///         client,
///     );
///     Ok(())
/// }
/// ```
pub struct GeminiEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    timeout: Duration,
    http_client: Client,
}

impl GeminiEmbeddingProvider {
    /// Create a new Gemini embedding provider
    ///
    /// # Arguments
    /// * `api_key` - Google AI API key
    /// * `base_url` - Optional custom base URL (defaults to Google AI API)
    /// * `model` - Model name (e.g., "text-embedding-004")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    // TODO(qlty): Found 17 lines of similar code in 4 locations (mass = 54)
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
        constructor::get_effective_url(
            self.base_url.as_deref(),
            "https://generativelanguage.googleapis.com",
        )
    }

    /// Get the model name for API calls (remove prefix if present)
    pub fn api_model_name(&self) -> &str {
        self.model.strip_prefix("models/").unwrap_or(&self.model)
    }

    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        match self.api_model_name() {
            "gemini-embedding-001" => 2048,
            "text-embedding-004" => 2048,
            _ => 2048,
        }
    }

    /// Get the API key for this provider
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> String {
        self.effective_base_url()
    }

    /// Fetch embedding for a single text
    async fn fetch_single_embedding(&self, text: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "content": { "parts": [{ "text": text }] }
        });

        let url = format!(
            "{}/v1beta/models/{}:embedContent",
            self.effective_base_url(),
            self.api_model_name()
        );

        let headers = vec![
            ("Content-Type", CONTENT_TYPE_JSON.to_string()),
            ("x-goog-api-key", self.api_key.clone()),
        ];

        send_json_request(JsonRequestParams {
            client: &self.http_client,
            method: reqwest::Method::POST,
            url,
            timeout: self.timeout,
            provider: "Gemini",
            operation: "embedContent",
            kind: RequestErrorKind::Embedding,
            headers: &headers,
            body: Some(&payload),
        })
        .await
    }

    /// Parse embedding from response data
    fn parse_embedding(&self, response_data: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = parse_float_array_lossy(
            response_data,
            "/embedding/values",
            "Invalid response format: missing embedding values",
        )?;

        let dimensions = embedding_vec.len();
        Ok(Embedding {
            vector: embedding_vec,
            model: self.model.clone(),
            dimensions,
        })
    }
}

#[async_trait]
/// Google Gemini implementation of the EmbeddingProvider trait.
impl EmbeddingProvider for GeminiEmbeddingProvider {
    /// Generates embeddings for a batch of texts.
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Gemini API doesn't support batch embedding - process sequentially
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let response_data = self.fetch_single_embedding(text).await?;
            results.push(self.parse_embedding(&response_data)?);
        }

        Ok(results)
    }

    /// Returns the embedding dimensions for the configured model.
    fn dimensions(&self) -> usize {
        match self.api_model_name() {
            "gemini-embedding-001" => EMBEDDING_DIMENSION_GEMINI,
            "text-embedding-004" => EMBEDDING_DIMENSION_GEMINI,
            _ => EMBEDDING_DIMENSION_GEMINI,
        }
    }

    /// Returns the provider name ("gemini").
    fn provider_name(&self) -> &str {
        "gemini"
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

/// Factory function for creating Gemini embedding provider instances.
fn gemini_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    use crate::utils::http::create_http_provider_config;

    let cfg = create_http_provider_config(config, "Gemini", "text-embedding-004")?;

    Ok(Arc::new(GeminiEmbeddingProvider::new(
        cfg.api_key,
        cfg.base_url,
        cfg.model,
        cfg.timeout,
        cfg.client,
    )))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static GEMINI_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "gemini",
    description: "Google Gemini embedding provider (gemini-embedding-001, text-embedding-004)",
    factory: gemini_factory,
};
