//! `VoyageAI` Embedding Provider
//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#embedding-providers)
//!
//! Implements the `EmbeddingProvider` port using `VoyageAI`'s embedding API.
//! Optimized for code embeddings with voyage-code-3 model.

use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use mcb_utils::constants::embedding::{
    EMBEDDING_DIMENSION_VOYAGEAI_CODE, EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
};
use reqwest::Client;

use crate::utils::embedding::{HttpEmbeddingClient, process_batch};

define_standard_embedding_provider! {
    struct_name: VoyageAIEmbeddingProvider,
    doc: "`VoyageAI` embedding provider — wraps the standard `/v1/embeddings` API with Bearer auth.",
    provider_name: "VoyageAI",
    provider_slug: "voyageai",
    base_url: mcb_utils::constants::embedding::VOYAGEAI_API_BASE_URL,
    max_tokens: mcb_utils::constants::embedding::VOYAGEAI_MAX_INPUT_TOKENS,
    dimensions: |model: &str| match model {
        "voyage-code-3" => EMBEDDING_DIMENSION_VOYAGEAI_CODE,
        _ => EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT,
    },
    factory_fn: voyageai_factory,
    static_name: VOYAGEAI_PROVIDER,
    description: "VoyageAI embedding provider (voyage-code-3, etc.)",
    config_name: "VoyageAI",
    default_model: "voyage-code-3",
}
