//! Gemini Embedding Provider
//!
//! Implements the `EmbeddingProvider` port using Google's Gemini embedding API.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::constants::embedding::EMBEDDING_DIMENSION_GEMINI;
use mcb_domain::constants::http::CONTENT_TYPE_JSON;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::utils::embedding::{HttpEmbeddingClient, parse_float_array_lossy};
use crate::utils::http::{JsonRequestParams, RequestErrorKind, send_json_request};
use crate::{define_http_embedding_provider, impl_http_provider_base, register_http_provider};

define_http_embedding_provider!(
    /// Gemini embedding provider
    ///
    /// Implements the `EmbeddingProvider` domain port using Google's Gemini embedding API.
    /// Receives HTTP client via constructor injection.
    GeminiEmbeddingProvider
);

impl_http_provider_base!(
    GeminiEmbeddingProvider,
    crate::constants::GEMINI_API_BASE_URL
);

impl GeminiEmbeddingProvider {
    /// Get the model name for API calls (remove prefix if present)
    #[must_use]
    pub fn api_model_name(&self) -> &str {
        self.client
            .model
            .strip_prefix("models/")
            .unwrap_or(&self.client.model)
    }

    /// Get the maximum tokens supported by this provider
    #[must_use]
    pub fn max_tokens(&self) -> usize {
        crate::constants::GEMINI_MAX_TOKENS
    }

    /// Get the API key for this provider
    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.client.api_key
    }

    /// Fetch embedding for a single text
    async fn fetch_single_embedding(&self, text: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "content": { "parts": [{ "text": text }] }
        });

        let url = format!(
            "{}/v1beta/models/{}:embedContent",
            self.client.base_url,
            self.api_model_name()
        );

        let headers = vec![
            ("Content-Type", CONTENT_TYPE_JSON.to_owned()),
            ("x-goog-api-key", self.client.api_key.clone()),
        ];

        send_json_request(JsonRequestParams {
            client: &self.client.client,
            method: reqwest::Method::POST,
            url,
            timeout: self.client.timeout,
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
            model: self.client.model.clone(),
            dimensions,
        })
    }
}

#[async_trait]
/// Google Gemini implementation of the `EmbeddingProvider` trait.
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
        EMBEDDING_DIMENSION_GEMINI
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

register_http_provider!(
    GeminiEmbeddingProvider,
    gemini_factory,
    GEMINI_PROVIDER,
    "gemini",
    "Google Gemini embedding provider (gemini-embedding-001, text-embedding-004)",
    "Gemini",
    "text-embedding-004"
);
