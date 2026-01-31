//! Common helpers for embedding providers
//!
//! Shared functionality and patterns used across multiple embedding
//! provider implementations to reduce code duplication (DRY principle).
//!
//! ## Available Helpers
//!
//! | Helper | Purpose |
//! |--------|---------|
//! | [`constructor`] | API key/URL validation, defaults |
//! | [`http`] | HTTP client creation with standard config |
//! | [`parsing`] | Embedding vector parsing from JSON |

use mcb_domain::error::{Error, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// Default timeout for embedding API requests (30 seconds)
pub const DEFAULT_EMBEDDING_TIMEOUT: Duration = Duration::from_secs(30);

/// Common constructor patterns used by embedding providers
///
/// Provides re-usable patterns for provider initialization.
pub mod constructor {
    use std::time::Duration;

    /// Template for validating and normalizing API keys
    pub fn validate_api_key(api_key: &str) -> String {
        api_key.trim().to_string()
    }

    /// Template for validating and normalizing URLs
    pub fn validate_url(url: Option<String>) -> Option<String> {
        url.map(|u| u.trim().to_string())
    }

    /// Template for default timeout when not specified
    pub fn default_timeout() -> Duration {
        Duration::from_secs(30)
    }

    /// Get effective URL with fallback to default
    ///
    /// Standardized approach for handling optional base URLs across all providers.
    pub fn get_effective_url(provided_url: Option<&str>, default_url: &str) -> String {
        provided_url
            .map(|url| url.trim().to_string())
            .unwrap_or_else(|| default_url.to_string())
    }
}

/// HTTP client helpers for embedding providers (DRY)
///
/// Provides standardized HTTP client creation and error handling
/// to eliminate duplication across provider factory functions.
pub mod http {
    use super::*;

    /// Create an HTTP client with standard timeout configuration
    ///
    /// # Arguments
    /// * `timeout_secs` - Request timeout in seconds
    ///
    /// # Returns
    /// Configured reqwest Client or error message
    ///
    /// # Example
    /// ```ignore
    /// use mcb_providers::embedding::helpers::http::create_client;
    ///
    /// let client = create_client(30)?;
    /// ```
    pub fn create_client(timeout_secs: u64) -> std::result::Result<Client, String> {
        Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))
    }

    /// Create an HTTP client with default 30-second timeout
    pub fn create_default_client() -> std::result::Result<Client, String> {
        create_client(30)
    }

    /// Handle reqwest errors with timeout detection
    ///
    /// # Arguments
    /// * `error` - The reqwest error
    /// * `timeout` - The configured timeout duration
    /// * `endpoint` - API endpoint for error context
    ///
    /// # Returns
    /// Appropriate domain Error with context
    pub fn handle_request_error(error: reqwest::Error, timeout: Duration, endpoint: &str) -> Error {
        if error.is_timeout() {
            Error::embedding(format!(
                "{} {:?}",
                crate::constants::ERROR_MSG_REQUEST_TIMEOUT,
                timeout
            ))
        } else {
            Error::embedding(format!("HTTP request to {endpoint} failed: {error}"))
        }
    }
}

/// Embedding vector parsing helpers (DRY)
///
/// Provides standardized JSON parsing for embedding responses
/// to eliminate duplication across providers.
pub mod parsing {
    use super::*;

    /// Parse embedding vector from JSON response
    ///
    /// Extracts float array from a JSON value at the specified field path.
    ///
    /// # Arguments
    /// * `item` - JSON object containing the embedding
    /// * `field` - Field name containing the vector (e.g., "embedding", "values")
    /// * `index` - Text index for error messages
    ///
    /// # Returns
    /// Vector of f32 values or parsing error
    ///
    /// # Example
    /// ```ignore
    /// use mcb_providers::embedding::helpers::parsing::parse_embedding_vector;
    ///
    /// let json = serde_json::json!({"embedding": [0.1, 0.2, 0.3]});
    /// let vec = parse_embedding_vector(&json, "embedding", 0)?;
    /// ```
    pub fn parse_embedding_vector(item: &Value, field: &str, index: usize) -> Result<Vec<f32>> {
        item.get(field)
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                Error::embedding(format!(
                    "Invalid embedding format for text {index}: missing or invalid '{field}' field"
                ))
            })?
            .iter()
            .map(|v| {
                v.as_f64().map(|f| f as f32).ok_or_else(|| {
                    Error::embedding(format!("Invalid number in embedding for text {index}"))
                })
            })
            .collect()
    }

    /// Parse embedding vector with nested path (e.g., "embedding.values")
    ///
    /// Handles nested JSON structures like Gemini's response format.
    pub fn parse_nested_embedding_vector(
        item: &Value,
        outer: &str,
        inner: &str,
        index: usize,
    ) -> Result<Vec<f32>> {
        let nested = item
            .get(outer)
            .ok_or_else(|| Error::embedding(format!("Missing '{outer}' field for text {index}")))?;
        parse_embedding_vector(nested, inner, index)
    }
}
