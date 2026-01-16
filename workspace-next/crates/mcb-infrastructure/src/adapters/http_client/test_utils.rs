//! Test Utilities for HTTP Client
//!
//! Provides null/mock implementations for testing providers
//! without actual network calls.

use reqwest::Client;
use std::time::Duration;

use super::{HttpClientConfig, HttpClientProvider};

/// Null HTTP client pool for testing
///
/// Returns a minimal client that will fail quickly on actual requests.
/// Useful for unit tests that mock HTTP responses at a higher level.
#[derive(Clone)]
pub struct NullHttpClientPool {
    config: HttpClientConfig,
    client: Client,
}

impl Default for NullHttpClientPool {
    fn default() -> Self {
        Self::new()
    }
}

impl NullHttpClientPool {
    /// Create a new null HTTP client pool for testing
    pub fn new() -> Self {
        // Create a minimal client with very short timeout
        let client = Client::builder()
            .timeout(Duration::from_millis(1))
            .build()
            .unwrap_or_default();

        Self {
            config: HttpClientConfig::default(),
            client,
        }
    }
}

impl HttpClientProvider for NullHttpClientPool {
    fn client(&self) -> &Client {
        &self.client
    }

    fn config(&self) -> &HttpClientConfig {
        &self.config
    }

    fn client_with_timeout(
        &self,
        _timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.client.clone())
    }

    fn is_enabled(&self) -> bool {
        false
    }
}

