//! Common helpers for embedding providers
//!
//! Shared functionality and patterns used across multiple embedding
//! provider implementations to reduce code duplication (DRY principle).
//!
//! ## Available Helpers
//!
//! | Helper | Purpose |
//! | -------- | --------- |
//! | [`constructor`] | API key/URL validation, defaults |

use crate::provider_utils::embedding_data_array;
use mcb_domain::error::Result;
use mcb_domain::value_objects::Embedding;
use reqwest::Client;
use std::time::Duration;

/// Common HTTP client state for embedding providers
pub struct HttpEmbeddingClient {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub timeout: Duration,
    pub client: Client,
}

impl HttpEmbeddingClient {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        default_base_url: &str,
        model: String,
        timeout: Duration,
        client: Client,
    ) -> Self {
        Self {
            api_key: api_key.trim().to_string(),
            base_url: base_url
                .map(|url| url.trim().to_string())
                .unwrap_or_else(|| default_base_url.to_string()),
            model,
            timeout,
            client,
        }
    }
}

/// Process a batch of texts using a fetch function and a parse function
pub async fn process_batch<F, P>(
    texts: &[String],
    fetch_fn: F,
    parse_fn: P,
) -> Result<Vec<Embedding>>
where
    F: std::future::Future<Output = Result<serde_json::Value>>,
    P: Fn(usize, &serde_json::Value) -> Result<Embedding>,
{
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let response_data = fetch_fn.await?;
    let data = embedding_data_array(&response_data, texts.len())?;

    data.iter()
        .enumerate()
        .map(|(i, item)| parse_fn(i, item))
        .collect()
}
