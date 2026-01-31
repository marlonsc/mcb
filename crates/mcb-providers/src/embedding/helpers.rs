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

use reqwest::Client;
use std::time::Duration;

/// Default timeout for embedding API requests (30 seconds)
pub(crate) const DEFAULT_EMBEDDING_TIMEOUT: Duration = Duration::from_secs(30);

/// Common constructor patterns used by embedding providers
///
/// Provides re-usable patterns for provider initialization.
pub(crate) mod constructor {

    /// Template for validating and normalizing API keys
    pub(crate) fn validate_api_key(api_key: &str) -> String {
        api_key.trim().to_string()
    }

    /// Template for validating and normalizing URLs
    pub(crate) fn validate_url(url: Option<String>) -> Option<String> {
        url.map(|u| u.trim().to_string())
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

/// HTTP client helpers for embedding providers (DRY)
///
/// Provides standardized HTTP client creation and error handling
/// to eliminate duplication across provider factory functions.
pub(crate) mod http {
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
    pub(crate) fn create_client(timeout_secs: u64) -> std::result::Result<Client, String> {
        Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))
    }

    /// Create an HTTP client with default 30-second timeout
    pub(crate) fn create_default_client() -> std::result::Result<Client, String> {
        create_client(30)
    }
}
