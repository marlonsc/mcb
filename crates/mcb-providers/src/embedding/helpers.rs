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
//! | [`http`] | HTTP client creation (re-exported from utils) |

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
