//! Ollama Embedding Provider
//!
//! Implements the `EmbeddingProvider` port using Ollama's local embedding API.
//! Supports various local embedding models like nomic-embed-text, all-minilm, etc.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::constants::embedding::{
    EMBEDDING_DIMENSION_OLLAMA_ARCTIC, EMBEDDING_DIMENSION_OLLAMA_DEFAULT,
    EMBEDDING_DIMENSION_OLLAMA_MINILM, EMBEDDING_DIMENSION_OLLAMA_MXBAI,
    EMBEDDING_DIMENSION_OLLAMA_NOMIC,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::constants::{EMBEDDING_OPERATION_NAME, EMBEDDING_PARAM_MODEL, HTTP_HEADER_CONTENT_TYPE};
use crate::utils::embedding::{HttpEmbeddingClient, parse_float_array_lossy};
use crate::utils::http::{JsonRequestParams, RequestErrorKind, send_json_request};
use mcb_domain::constants::http::CONTENT_TYPE_JSON;

use crate::define_http_embedding_provider;

define_http_embedding_provider!(
    /// Ollama embedding provider
    ///
    /// Implements the `EmbeddingProvider` domain port using Ollama's local embedding API.
    /// Receives HTTP client via constructor injection.
    OllamaEmbeddingProvider
);

impl OllamaEmbeddingProvider {
    /// Create a new Ollama embedding provider
    ///
    /// # Arguments
    /// * `base_url` - Ollama server URL (e.g., "<http://localhost:11434>")
    /// * `model` - Model name (e.g., "nomic-embed-text")
    /// * `timeout` - Request timeout duration
    /// * `http_client` - Reqwest HTTP client for making API requests
    #[must_use]
    pub fn new(base_url: String, model: String, timeout: Duration, http_client: Client) -> Self {
        Self {
            client: HttpEmbeddingClient::new(
                "", // No API key for Ollama
                Some(base_url),
                crate::constants::OLLAMA_DEFAULT_BASE_URL,
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
        match self.client.model.as_str() {
            "all-minilm" | "mxbai-embed-large" | "snowflake-arctic-embed" => {
                crate::constants::OLLAMA_MAX_TOKENS_LIMITED
            }
            _ => crate::constants::OLLAMA_MAX_TOKENS_DEFAULT,
        }
    }

    /// Fetch embedding for a single text
    async fn fetch_single_embedding(&self, text: &str) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            (EMBEDDING_PARAM_MODEL): self.client.model,
            "prompt": text,
            "stream": false
        });

        let headers = vec![(HTTP_HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON.to_owned())];

        send_json_request(JsonRequestParams {
            client: &self.client.client,
            method: reqwest::Method::POST,
            url: format!(
                "{}/api/embeddings",
                self.client.base_url.trim_end_matches('/')
            ),
            timeout: self.client.timeout,
            provider: "Ollama",
            operation: EMBEDDING_OPERATION_NAME,
            kind: RequestErrorKind::Embedding,
            headers: &headers,
            body: Some(&payload),
            retry: None,
        })
        .await
        .map_err(|e| {
            if let Error::Embedding { message, .. } = &e
                && message.contains("HTTP request to Ollama failed")
            {
                return Error::embedding(
                    message.replace("HTTP request to Ollama failed", "HTTP request failed"),
                );
            }
            e
        })
    }

    /// Parse embedding from response data
    fn parse_embedding(&self, response_data: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = parse_float_array_lossy(
            response_data,
            "/embedding",
            "Invalid response format: missing embedding array",
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
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Ollama API doesn't support batch embedding - process sequentially
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let response_data = self.fetch_single_embedding(text).await?;
            results.push(self.parse_embedding(&response_data)?);
        }

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        match self.client.model.as_str() {
            "nomic-embed-text" => EMBEDDING_DIMENSION_OLLAMA_NOMIC,
            "all-minilm" => EMBEDDING_DIMENSION_OLLAMA_MINILM,
            "mxbai-embed-large" => EMBEDDING_DIMENSION_OLLAMA_MXBAI,
            "snowflake-arctic-embed" => EMBEDDING_DIMENSION_OLLAMA_ARCTIC,
            _ => EMBEDDING_DIMENSION_OLLAMA_DEFAULT,
        }
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::ports::EmbeddingProvider as EmbeddingProviderPort;
use mcb_domain::registry::embedding::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};

/// Factory function for creating Ollama embedding provider instances.
fn ollama_factory(
    config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    use crate::utils::http::{DEFAULT_HTTP_TIMEOUT, create_default_client};

    let base_url = config
        .base_url
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{}", crate::constants::OLLAMA_DEFAULT_PORT));
    let model = config
        .model
        .clone()
        .unwrap_or_else(|| crate::constants::OLLAMA_DEFAULT_MODEL.to_owned());
    let http_client = create_default_client()?;

    Ok(Arc::new(OllamaEmbeddingProvider::new(
        base_url,
        model,
        DEFAULT_HTTP_TIMEOUT,
        http_client,
    )))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    description: "Ollama local embedding provider (nomic-embed-text, all-minilm, etc.)",
    build: ollama_factory,
};
