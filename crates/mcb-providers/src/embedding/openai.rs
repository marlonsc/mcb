//! `OpenAI` Embedding Provider
//!
//! Implements the `EmbeddingProvider` port using `OpenAI`'s embedding API.
//! Supports text-embedding-3-small, text-embedding-3-large, and ada-002.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::constants::embedding::{
    EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_LARGE,
    EMBEDDING_DIMENSION_OPENAI_SMALL,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;

use crate::utils::embedding::{HttpEmbeddingClient, process_batch};
use crate::utils::http::{
    JsonRequestParams, RequestErrorKind, parse_embedding_vector, send_json_request,
};
use crate::{
    define_http_embedding_provider, impl_embedding_provider_trait, impl_http_provider_base,
    register_http_provider,
};

use mcb_domain::constants::http::CONTENT_TYPE_JSON;

define_http_embedding_provider!(
    /// `OpenAI` embedding provider
    ///
    /// Implements the `EmbeddingProvider` domain port using `OpenAI`'s embedding API.
    /// Receives HTTP client via constructor injection.
    OpenAIEmbeddingProvider
);

impl_http_provider_base!(
    OpenAIEmbeddingProvider,
    crate::constants::OPENAI_API_BASE_URL
);

impl OpenAIEmbeddingProvider {
    /// Get the maximum tokens for this model
    #[must_use]
    pub fn max_tokens(&self) -> usize {
        crate::constants::OPENAI_MAX_TOKENS_PER_REQUEST
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
            ("Content-Type", CONTENT_TYPE_JSON.to_owned()),
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

impl_embedding_provider_trait!(
    OpenAIEmbeddingProvider,
    "openai",
    |model: &str| match model {
        "text-embedding-3-large" => EMBEDDING_DIMENSION_OPENAI_LARGE,
        "text-embedding-ada-002" => EMBEDDING_DIMENSION_OPENAI_ADA,
        _ => EMBEDDING_DIMENSION_OPENAI_SMALL,
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
    OpenAIEmbeddingProvider,
    openai_factory,
    OPENAI_PROVIDER,
    "openai",
    "OpenAI embedding provider (text-embedding-3-small/large, ada-002)",
    "OpenAI",
    "text-embedding-3-small"
);
