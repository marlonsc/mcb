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
//! | [`http`] | HTTP client creation (re-exported from utils) |

/// Common constructor patterns used by embedding providers
///
/// Provides re-usable patterns for provider initialization.
pub(crate) mod constructor {
    /// Template for validating and normalizing API keys
    pub(crate) fn validate_api_key(api_key: &str) -> String {
        api_key.trim().to_string()
    }

    /// Get effective URL with fallback to default
    ///
    /// Standardized approach for handling optional base URLs across all providers.
    pub(crate) fn get_effective_url(provided_url: Option<&str>, default_url: &str) -> String {
        provided_url
            .map(|url| url.trim().to_string())
            .unwrap_or_else(|| default_url.to_string())
    }
}

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
            api_key: constructor::validate_api_key(&api_key),
            base_url: constructor::get_effective_url(base_url.as_deref(), default_base_url),
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
