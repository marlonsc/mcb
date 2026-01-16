//! VoyageAI Embedding Provider
//!
//! Implements the EmbeddingProvider port using VoyageAI's embedding API.
//! Optimized for code embeddings with voyage-code-3 model.

use std::sync::Arc;

use async_trait::async_trait;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::adapters::http_client::HttpClientProvider;
use crate::adapters::providers::embedding::helpers::constructor;
use crate::constants::{EMBEDDING_DIMENSION_VOYAGEAI_CODE, EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT};
use crate::utils::HttpResponseUtils;

/// VoyageAI embedding provider
///
/// Implements the `EmbeddingProvider` domain port using VoyageAI's embedding API.
/// Receives HTTP client via constructor injection for DI compliance.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_infrastructure::adapters::providers::embedding::VoyageAIEmbeddingProvider;
/// use mcb_infrastructure::adapters::http_client::HttpClientPool;
/// use std::sync::Arc;
///
/// let http_client = Arc::new(HttpClientPool::new().unwrap());
/// let provider = VoyageAIEmbeddingProvider::new(
///     "voyage-xxx".to_string(),
///     None,
///     "voyage-code-3".to_string(),
///     http_client,
/// );
/// ```
pub struct VoyageAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
    http_client: Arc<dyn HttpClientProvider>,
}

impl VoyageAIEmbeddingProvider {
    /// Create a new VoyageAI embedding provider
    ///
    /// # Arguments
    /// * `api_key` - VoyageAI API key
    /// * `base_url` - Optional custom base URL (defaults to VoyageAI API)
    /// * `model` - Model name (e.g., "voyage-code-3")
    /// * `http_client` - Injected HTTP client (required for DI compliance)
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: String,
        http_client: Arc<dyn HttpClientProvider>,
    ) -> Self {
        let api_key = constructor::validate_api_key(&api_key);
        let base_url = constructor::validate_url(base_url);
        Self {
            api_key,
            base_url,
            model,
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
        match self.model.as_str() {
            "voyage-code-3" => 16000,
            _ => 16000,
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
}

#[async_trait]
impl EmbeddingProvider for VoyageAIEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let payload = serde_json::json!({
            "input": texts,
            "model": self.model
        });

        let client = self.http_client.client();
        let base_url = self.effective_base_url();

        let response = client
            .post(format!("{}/embeddings", base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::embedding(format!("HTTP request failed: {}", e)))?;

        let response_data: serde_json::Value =
            HttpResponseUtils::check_and_parse(response, "VoyageAI").await?;

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

        let embeddings = data
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let embedding_vec = item["embedding"]
                    .as_array()
                    .ok_or_else(|| {
                        Error::embedding(format!("Invalid embedding format for text {}", i))
                    })?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<f32>>();

                Ok(Embedding {
                    vector: embedding_vec,
                    model: self.model.clone(),
                    dimensions: self.dimensions(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(embeddings)
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

