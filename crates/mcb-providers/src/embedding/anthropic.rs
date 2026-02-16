//! Anthropic embedding provider backed by Voyage models.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::constants::embedding::{
    EMBEDDING_DIMENSION_ANTHROPIC_CODE, EMBEDDING_DIMENSION_ANTHROPIC_DEFAULT,
    EMBEDDING_DIMENSION_ANTHROPIC_LITE,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::constants::ANTHROPIC_MAX_INPUT_TOKENS;
use crate::{
    define_http_embedding_provider, impl_embedding_provider_trait, impl_http_provider_base,
    register_http_provider,
};

use crate::constants::{
    EMBEDDING_API_ENDPOINT, EMBEDDING_OPERATION_NAME, EMBEDDING_PARAM_INPUT, EMBEDDING_PARAM_MODEL,
    EMBEDDING_RESPONSE_FIELD, HTTP_HEADER_AUTHORIZATION, HTTP_HEADER_CONTENT_TYPE,
};
use crate::utils::embedding::{HttpEmbeddingClient, process_batch};
use crate::utils::http::{
    JsonRequestParams, RequestErrorKind, RetryConfig, parse_embedding_vector, send_json_request,
};
use mcb_domain::constants::http::CONTENT_TYPE_JSON;

define_http_embedding_provider!(
    /// Anthropic embedding provider
    ///
    /// Implements the `EmbeddingProvider` domain port using Anthropic's embedding API.
    /// Uses Voyage AI models accessed through the Anthropic API endpoint.
    /// Receives HTTP client via constructor injection.
    AnthropicEmbeddingProvider
);

impl_http_provider_base!(
    AnthropicEmbeddingProvider,
    crate::constants::VOYAGEAI_API_BASE_URL
);

impl AnthropicEmbeddingProvider {
    /// Get the maximum tokens for this model
    #[must_use]
    pub fn max_tokens(&self) -> usize {
        // All Voyage AI models support the same max tokens
        ANTHROPIC_MAX_INPUT_TOKENS
    }

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            (EMBEDDING_PARAM_INPUT): texts,
            (EMBEDDING_PARAM_MODEL): self.client.model,
            "input_type": "document",
            "encoding_format": "float"
        });

        let headers = vec![
            (
                HTTP_HEADER_AUTHORIZATION,
                format!("Bearer {}", self.client.api_key),
            ),
            (HTTP_HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON.to_owned()),
        ];

        send_json_request(JsonRequestParams {
            client: &self.client.client,
            method: reqwest::Method::POST,
            url: format!("{}{}", self.base_url(), EMBEDDING_API_ENDPOINT),
            timeout: self.client.timeout,
            provider: "Anthropic",
            operation: EMBEDDING_OPERATION_NAME,
            kind: RequestErrorKind::Embedding,
            headers: &headers,
            body: Some(&payload),
            retry: Some(RetryConfig::new(3, std::time::Duration::from_millis(500))),
        })
        .await
    }

    /// Parse embedding vector from response data
    fn parse_embedding(&self, index: usize, item: &serde_json::Value) -> Result<Embedding> {
        let embedding_vec = parse_embedding_vector(item, EMBEDDING_RESPONSE_FIELD, index)?;

        Ok(Embedding {
            vector: embedding_vec,
            model: self.client.model.clone(),
            dimensions: self.dimensions(),
        })
    }
}

impl_embedding_provider_trait!(
    AnthropicEmbeddingProvider,
    "anthropic",
    |model: &str| match model {
        "voyage-3-lite" => EMBEDDING_DIMENSION_ANTHROPIC_LITE,
        "voyage-code-3" => EMBEDDING_DIMENSION_ANTHROPIC_CODE,
        _ => EMBEDDING_DIMENSION_ANTHROPIC_DEFAULT,
    }
);

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;
use mcb_domain::registry::embedding::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};

register_http_provider!(
    AnthropicEmbeddingProvider,
    anthropic_factory,
    ANTHROPIC_PROVIDER,
    "anthropic",
    "Anthropic embedding provider (voyage-3, voyage-3-lite, voyage-code-3)",
    "Anthropic",
    "voyage-3"
);
