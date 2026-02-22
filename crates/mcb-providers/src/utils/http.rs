//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! HTTP Client Utilities
//!
//! Shared HTTP client creation and error handling utilities
//! used across embedding and vector store providers (DRY principle).

use std::time::Duration;

use mcb_domain::error::Error;
use reqwest::Client;
use serde_json::Value;

use super::http_response::HttpResponseUtils;
use super::retry::retry_with_backoff;
use crate::constants::ERROR_MSG_REQUEST_TIMEOUT;

// Re-export so callers of `send_json_request` can build `JsonRequestParams.retry`.
pub(crate) use super::retry::RetryConfig;

/// Default timeout for HTTP requests (30 seconds)
pub(crate) const DEFAULT_HTTP_TIMEOUT: Duration =
    Duration::from_secs(crate::constants::DEFAULT_HTTP_TIMEOUT_SECS);

#[derive(Debug, Clone, Copy)]
/// Classification used to map HTTP request failures to domain errors.
pub(crate) enum RequestErrorKind {
    /// Embedding provider request.
    Embedding,
    /// Vector database provider request.
    VectorDb,
}

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
pub(crate) fn create_client(timeout_secs: u64) -> std::result::Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

/// Create an HTTP client with the default 30-second timeout
pub(crate) fn create_default_client() -> std::result::Result<Client, String> {
    create_client(30)
}

pub(crate) fn handle_request_error_with_kind(
    error: &reqwest::Error,
    timeout: Duration,
    provider: &str,
    operation: &str,
    kind: RequestErrorKind,
) -> Error {
    match kind {
        RequestErrorKind::Embedding => {
            if error.is_timeout() {
                Error::embedding(format!("{ERROR_MSG_REQUEST_TIMEOUT} {timeout:?}"))
            } else {
                Error::embedding(format!("HTTP request to {provider} failed: {error}"))
            }
        }
        RequestErrorKind::VectorDb => {
            if error.is_timeout() {
                Error::vector_db(format!(
                    "{provider} {operation} request timed out after {timeout:?}"
                ))
            } else {
                Error::vector_db(format!(
                    "{provider} HTTP request for {operation} failed: {error}"
                ))
            }
        }
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
) -> std::result::Result<HttpProviderConfig, String> {
    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| format!("{provider_name} requires api_key"))?;

    let base_url = config.base_url.clone();

    let model = config
        .model
        .clone()
        .unwrap_or_else(|| default_model.to_owned());

    let client = create_default_client()?;

    Ok(HttpProviderConfig {
        api_key,
        base_url,
        model,
        timeout: DEFAULT_HTTP_TIMEOUT,
        client,
    })
}

/// Parameters for [`send_json_request`].
pub(crate) struct JsonRequestParams<'a> {
    /// HTTP client to use.
    pub client: &'a Client,
    /// HTTP method (GET, POST, etc.).
    pub method: reqwest::Method,
    /// Target URL.
    pub url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Provider name for error messages.
    pub provider: &'a str,
    /// Operation name for error messages.
    pub operation: &'a str,
    /// Error classification kind.
    pub kind: RequestErrorKind,
    /// Additional headers.
    pub headers: &'a [(&'a str, String)],
    /// Optional JSON body.
    pub body: Option<&'a Value>,
    /// Optional retry configuration for transient errors (rate limits, 5xx, timeouts).
    pub retry: Option<RetryConfig>,
}

/// Check whether a domain error represents a transient HTTP failure worth retrying.
fn is_retryable_error(error: &Error) -> bool {
    let msg = error.to_string();
    msg.contains("rate limit exceeded")
        || msg.contains("server error (5")
        || msg.contains("timed out")
        || msg.contains("timeout")
}

/// Send a JSON request with configurable parameters and optional retry.
pub(crate) async fn send_json_request(
    params: JsonRequestParams<'_>,
) -> mcb_domain::error::Result<Value> {
    let JsonRequestParams {
        client,
        method,
        url,
        timeout,
        provider,
        operation,
        kind,
        headers,
        body,
        retry,
    } = params;

    let execute = || async {
        let mut builder = client.request(method.clone(), &url).timeout(timeout);

        for (key, value) in headers {
            builder = builder.header(*key, value);
        }

        if let Some(payload) = body {
            builder = builder.json(payload);
        }

        let response = builder
            .send()
            .await
            .map_err(|e| handle_request_error_with_kind(&e, timeout, provider, operation, kind))?;

        HttpResponseUtils::check_and_parse(response, provider).await
    };

    match retry {
        None => execute().await,
        Some(config) => retry_with_backoff(config, |_| execute(), is_retryable_error).await,
    }
}

pub(crate) struct VectorDbRequestParams<'a> {
    pub client: &'a Client,
    pub method: reqwest::Method,
    pub url: String,
    pub timeout: Duration,
    pub provider: &'a str,
    pub operation: &'a str,
    pub headers: &'a [(&'a str, String)],
    pub body: Option<&'a Value>,
    pub retry_attempts: usize,
    pub retry_backoff_secs: u64,
}

pub(crate) async fn send_vector_db_request(
    params: VectorDbRequestParams<'_>,
) -> mcb_domain::error::Result<Value> {
    let VectorDbRequestParams {
        client,
        method,
        url,
        timeout,
        provider,
        operation,
        headers,
        body,
        retry_attempts,
        retry_backoff_secs,
    } = params;

    send_json_request(JsonRequestParams {
        client,
        method,
        url,
        timeout,
        provider,
        operation,
        kind: RequestErrorKind::VectorDb,
        headers,
        body,
        retry: Some(RetryConfig::new(
            retry_attempts,
            Duration::from_secs(retry_backoff_secs),
        )),
    })
    .await
}
