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

use crate::constants::VOYAGEAI_MAX_INPUT_TOKENS;
use crate::utils::embedding::{HttpEmbeddingClient, process_batch};
use crate::{
    define_http_embedding_provider, impl_embedding_provider_trait, impl_http_provider_base,
    register_http_provider,
};

use crate::constants::{
    EMBEDDING_API_ENDPOINT, EMBEDDING_OPERATION_NAME, HTTP_HEADER_AUTHORIZATION,
    HTTP_HEADER_CONTENT_TYPE,
};
use crate::utils::http::{
    JsonRequestParams, RequestErrorKind, RetryConfig, parse_embedding_vector, send_json_request,
};
use mcb_domain::constants::http::CONTENT_TYPE_JSON;

define_http_embedding_provider!(
    /// `VoyageAI` embedding provider
    ///
    /// Implements the `EmbeddingProvider` domain port using `VoyageAI`'s embedding API.
    /// Receives HTTP client via constructor injection.
    VoyageAIEmbeddingProvider
);

impl_http_provider_base!(
    VoyageAIEmbeddingProvider,
    crate::constants::VOYAGEAI_API_BASE_URL
);

impl VoyageAIEmbeddingProvider {
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

    /// Send embedding request and get response data
    async fn fetch_embeddings(&self, texts: &[String]) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "input": texts,
            "model": self.client.model
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
            url: format!("{}{}", self.client.base_url, EMBEDDING_API_ENDPOINT),
            timeout: self.client.timeout,
            provider: "VoyageAI",
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
        let embedding_vec = parse_embedding_vector(item, "embedding", index)?;

        Ok(Embedding {
            vector: embedding_vec,
            model: self.client.model.clone(),
            dimensions: self.dimensions(),
        })
    }
}

impl_embedding_provider_trait!(
    VoyageAIEmbeddingProvider,
    "voyageai",
    |model: &str| match model {
        "voyage-code-3" => EMBEDDING_DIMENSION_VOYAGEAI_CODE,
        _ => EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
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
    VoyageAIEmbeddingProvider,
    voyageai_factory,
    VOYAGEAI_PROVIDER,
    "voyageai",
    "VoyageAI embedding provider (voyage-code-3, etc.)",
    "VoyageAI",
    "voyage-code-3"
);
