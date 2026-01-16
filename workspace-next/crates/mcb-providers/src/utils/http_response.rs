//! HTTP Response Utilities
//!
//! Helper functions for processing HTTP responses from API providers.
//! These are shared utilities, not ports.

use mcb_domain::error::{Error, Result};
use reqwest::Response;

/// Utilities for processing HTTP responses
///
/// Provides common response handling patterns used by embedding providers.
pub struct HttpResponseUtils;

impl HttpResponseUtils {
    /// Check response status and parse JSON
    ///
    /// # Arguments
    /// * `response` - The HTTP response to check
    /// * `provider_name` - Name of the provider for error messages
    ///
    /// # Returns
    /// Parsed JSON value on success, or an appropriate error
    pub async fn check_and_parse(
        response: Response,
        provider_name: &str,
    ) -> Result<serde_json::Value> {
        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            return match status.as_u16() {
                401 => Err(Error::embedding(format!(
                    "{} authentication failed: {}",
                    provider_name, error_text
                ))),
                429 => Err(Error::embedding(format!(
                    "{} rate limit exceeded: {}",
                    provider_name, error_text
                ))),
                500..=599 => Err(Error::embedding(format!(
                    "{} server error ({}): {}",
                    provider_name,
                    status.as_u16(),
                    error_text
                ))),
                _ => Err(Error::embedding(format!(
                    "{} request failed ({}): {}",
                    provider_name,
                    status.as_u16(),
                    error_text
                ))),
            };
        }

        response.json().await.map_err(|e| {
            Error::embedding(format!("Failed to parse {} response: {}", provider_name, e))
        })
    }
}
