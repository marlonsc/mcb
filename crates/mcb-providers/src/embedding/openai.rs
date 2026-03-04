//! `OpenAI` Embedding Provider
//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#embedding-providers)
//!
//! Implements the `EmbeddingProvider` port using `OpenAI`'s embedding API.
//! Supports text-embedding-3-small, text-embedding-3-large, and ada-002.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use mcb_utils::constants::embedding::{
    EMBEDDING_DIMENSION_OPENAI_ADA, EMBEDDING_DIMENSION_OPENAI_LARGE,
    EMBEDDING_DIMENSION_OPENAI_SMALL,
};
use reqwest::Client;

use crate::utils::embedding::{HttpEmbeddingClient, process_batch};

define_standard_embedding_provider! {
    struct_name: OpenAIEmbeddingProvider,
    doc: "OpenAI embedding provider — wraps the standard /v1/embeddings API with Bearer auth.",
    provider_name: "OpenAI",
    provider_slug: "openai",
    base_url: mcb_utils::constants::embedding::OPENAI_API_BASE_URL,
    max_tokens: mcb_utils::constants::embedding::OPENAI_MAX_TOKENS_PER_REQUEST,
    dimensions: |model: &str| match model {
        "text-embedding-3-large" => EMBEDDING_DIMENSION_OPENAI_LARGE,
        "text-embedding-ada-002" => EMBEDDING_DIMENSION_OPENAI_ADA,
        _ => EMBEDDING_DIMENSION_OPENAI_SMALL,
    },
    extra_payload: { "encoding_format": "float" },
    factory_fn: openai_factory,
    static_name: OPENAI_PROVIDER,
    description: "OpenAI embedding provider (text-embedding-3-small/large, ada-002)",
    config_name: "OpenAI",
    default_model: "text-embedding-3-small",
}
