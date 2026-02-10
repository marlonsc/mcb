//! HTTP Client Utilities
//!
//! Shared HTTP client creation and error handling utilities
//! used across embedding and vector store providers (DRY principle).

use std::time::Duration;

use mcb_domain::error::Error;
use reqwest::Client;

use crate::constants::ERROR_MSG_REQUEST_TIMEOUT;

/// Default timeout for HTTP requests (30 seconds)
pub(crate) const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// Create an HTTP client with the specified timeout
///
/// # Arguments
/// * `timeout_secs` - Request timeout in seconds
///
/// # Returns
/// Configured reqwest Client or error message string
///
/// # Example
/// ```ignore
/// use mcb_providers::utils::http::create_client;
///
/// let client = create_client(30)?;
/// ```
pub(crate) fn create_client(timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

/// Create an HTTP client with the default 30-second timeout
pub(crate) fn create_default_client() -> Result<Client, String> {
    create_client(30)
}

/// Handle HTTP request errors with proper timeout detection
///
/// Converts reqwest errors into domain errors with appropriate
/// messages for timeouts vs other failures.
///
/// # Arguments
/// * `error` - The reqwest error to handle
/// * `timeout` - The timeout duration for error messages
/// * `endpoint` - The endpoint name for error context
///
/// # Returns
/// Domain Error with appropriate message
pub(crate) fn handle_request_error(
    error: reqwest::Error,
    timeout: Duration,
    endpoint: &str,
) -> Error {
    if error.is_timeout() {
        Error::embedding(format!("{} {:?}", ERROR_MSG_REQUEST_TIMEOUT, timeout))
    } else {
        Error::embedding(format!("HTTP request to {endpoint} failed: {error}"))
    }
}

/// Parse embedding vector from JSON response item
///
/// Extracts embedding vector from a JSON value, handling errors appropriately.
///
/// # Arguments
/// * `item` - JSON value containing the embedding
/// * `field` - Field name to extract (e.g., "embedding", "values")
/// * `index` - Text index for error messages
///
/// # Returns
/// Parsed embedding vector or domain error
pub(crate) fn parse_embedding_vector(
    item: &serde_json::Value,
    field: &str,
    index: usize,
) -> mcb_domain::error::Result<Vec<f32>> {
    item[field]
        .as_array()
        .ok_or_else(|| Error::embedding(format!("Invalid embedding format for text {index}")))?
        .iter()
        .map(|v| {
            v.as_f64().map(|n| n as f32).ok_or_else(|| {
                Error::embedding(format!("Invalid number in embedding for text {index}"))
            })
        })
        .collect()
}

/// Configuration for HTTP-based embedding providers
///
/// Extracted from `EmbeddingProviderConfig` with provider-specific defaults applied.
pub(crate) struct HttpProviderConfig {
    /// API key for authentication (required for most providers)
    pub api_key: String,
    /// Base URL for the API endpoint
    pub base_url: Option<String>,
    /// Model name to use
    pub model: String,
    /// Request timeout
    pub timeout: Duration,
    /// Configured HTTP client
    pub client: Client,
}

/// Create configuration for an HTTP-based embedding provider
///
/// This helper extracts common configuration from `EmbeddingProviderConfig`,
/// applies provider-specific defaults, and creates the HTTP client.
///
/// # Arguments
/// * `config` - The provider configuration from registry
/// * `provider_name` - Name of the provider (for error messages)
/// * `default_model` - Default model to use if none specified
///
/// # Returns
/// Configured `HttpProviderConfig` or error message
///
/// # Example
/// ```ignore
/// let cfg = create_http_provider_config(config, "openai", "text-embedding-3-small")?;
/// Ok(Arc::new(OpenAIEmbeddingProvider::new(
///     cfg.api_key,
///     cfg.base_url,
///     cfg.model,
///     cfg.timeout,
///     cfg.client,
/// )))
/// ```
pub(crate) fn create_http_provider_config(
    config: &mcb_domain::registry::embedding::EmbeddingProviderConfig,
    provider_name: &str,
    default_model: &str,
) -> Result<HttpProviderConfig, String> {
    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| format!("{provider_name} requires api_key"))?;

    let base_url = config.base_url.clone();

    let model = config
        .model
        .clone()
        .unwrap_or_else(|| default_model.to_string());

    let client = create_default_client()?;

    Ok(HttpProviderConfig {
        api_key,
        base_url,
        model,
        timeout: DEFAULT_HTTP_TIMEOUT,
        client,
    })
}
