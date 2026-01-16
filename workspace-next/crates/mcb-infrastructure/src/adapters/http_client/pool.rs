//! HTTP Client Pool Implementation
//!
//! Thread-safe HTTP client pool with connection reuse for performance.

use reqwest::Client;
use std::time::Duration;

use super::{HttpClientConfig, HttpClientProvider};

/// Thread-safe HTTP client pool
///
/// Provides connection pooling and reuse for HTTP-based providers
/// like embedding services and vector stores.
///
/// ## Example
///
/// ```rust,no_run
/// use mcb_infrastructure::adapters::http_client::{HttpClientPool, HttpClientProvider};
///
/// let pool = HttpClientPool::new().expect("Failed to create pool");
/// let client = pool.client();
/// // Use client for API calls
/// ```
#[derive(Clone)]
pub struct HttpClientPool {
    /// The underlying reqwest HTTP client with connection pooling
    client: Client,
    /// Configuration for the HTTP client pool
    config: HttpClientConfig,
}

impl HttpClientPool {
    /// Create a new HTTP client pool with default configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_config(HttpClientConfig::default())
    }

    /// Create a new HTTP client pool with custom configuration
    pub fn with_config(
        config: HttpClientConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Self::build_client(&config, config.timeout)?;
        Ok(Self { client, config })
    }

    /// Build a reqwest Client from config (DRY helper)
    fn build_client(
        config: &HttpClientConfig,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        Client::builder()
            .pool_max_idle_per_host(config.max_idle_per_host)
            .pool_idle_timeout(config.idle_timeout)
            .tcp_keepalive(config.keepalive)
            .timeout(timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl HttpClientProvider for HttpClientPool {
    fn client(&self) -> &Client {
        &self.client
    }

    fn config(&self) -> &HttpClientConfig {
        &self.config
    }

    fn client_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        Self::build_client(&self.config, timeout)
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

