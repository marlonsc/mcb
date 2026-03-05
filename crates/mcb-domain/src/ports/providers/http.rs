//! HTTP client provider ports.

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// HTTP client configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Maximum number of idle connections per host.
    pub max_idle_per_host: usize,
    /// Maximum idle time for a connection.
    pub idle_timeout: Duration,
    /// Connection keep-alive interval.
    pub keepalive: Duration,
    /// Request timeout.
    pub timeout: Duration,
    /// User agent string for outgoing requests.
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            keepalive: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            user_agent: "mcb/domain-client".to_owned(),
        }
    }
}

/// HTTP client provider trait.
///
/// Provides an abstract HTTP client interface without coupling to any
/// concrete HTTP library. Implementations live in the provider or
/// infrastructure layer and wrap a real HTTP client.
#[async_trait]
pub trait HttpClientProvider: Send + Sync {
    /// Get the HTTP client configuration.
    fn config(&self) -> &HttpClientConfig;

    /// Execute a GET request and return the response body as bytes.
    ///
    /// # Errors
    /// Returns an error if the request fails.
    async fn get(&self, url: &str) -> Result<Vec<u8>>;

    /// Execute a POST request with a JSON body and return the response body.
    ///
    /// # Errors
    /// Returns an error if the request fails.
    async fn post(&self, url: &str, body: &[u8]) -> Result<Vec<u8>>;

    /// Return true if the HTTP client is configured and enabled.
    fn is_enabled(&self) -> bool;
}
