//! HTTP Client Utilities
//!
//! Shared HTTP client creation and error handling utilities
//! used across embedding and vector store providers (DRY principle).

use std::time::Duration;

use mcb_domain::error::Error;
use reqwest::Client;

use crate::constants::ERROR_MSG_REQUEST_TIMEOUT;

/// Default timeout for HTTP requests (30 seconds)
#[allow(dead_code)] // Reserved for future use
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
