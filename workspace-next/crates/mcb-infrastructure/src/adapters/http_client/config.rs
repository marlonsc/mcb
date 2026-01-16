//! HTTP Client Configuration
//!
//! Configuration struct for HTTP client pool settings.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::constants::{
    HTTP_CLIENT_IDLE_TIMEOUT_SECS, HTTP_KEEPALIVE_SECS, HTTP_MAX_IDLE_PER_HOST,
    HTTP_REQUEST_TIMEOUT_SECS,
};

/// HTTP client pool configuration
///
/// Controls connection pooling, timeouts, and other HTTP client behavior.
/// Sensible defaults are provided based on infrastructure constants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    /// Maximum idle connections per host
    pub max_idle_per_host: usize,
    /// Idle connection timeout
    pub idle_timeout: Duration,
    /// TCP keep-alive duration
    pub keepalive: Duration,
    /// Total timeout for requests
    pub timeout: Duration,
    /// User agent string
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: HTTP_MAX_IDLE_PER_HOST,
            idle_timeout: Duration::from_secs(HTTP_CLIENT_IDLE_TIMEOUT_SECS),
            keepalive: Duration::from_secs(HTTP_KEEPALIVE_SECS),
            timeout: Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECS),
            user_agent: format!("MCP-Context-Browser/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl HttpClientConfig {
    /// Create a new configuration with custom settings
    pub fn new(
        max_idle_per_host: usize,
        idle_timeout: Duration,
        keepalive: Duration,
        timeout: Duration,
        user_agent: String,
    ) -> Self {
        Self {
            max_idle_per_host,
            idle_timeout,
            keepalive,
            timeout,
            user_agent,
        }
    }

    /// Create configuration with custom timeout only
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }
}
