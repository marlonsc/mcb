//! HTTP Client Provider Trait
//!
//! Defines the interface for HTTP client operations used by API-based providers.
//! This trait enables dependency injection for HTTP-based adapters.

mod types;
pub use types::*;

use reqwest::Client;
use std::time::Duration;

/// HTTP client provider trait
///
/// Defines the interface for HTTP client operations used by API-based providers.
pub trait HttpClientProvider: Send + Sync {
    /// Get a reference to the underlying reqwest Client
    fn client(&self) -> &Client;

    /// Get the configuration
    fn config(&self) -> &HttpClientConfig;

    /// Create a new client with custom timeout for specific operations
    fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>>;

    /// Check if the client pool is enabled
    fn is_enabled(&self) -> bool;
}
