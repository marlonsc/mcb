use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP client configuration
///
/// Controls connection pooling, timeouts, and other HTTP client behavior.
/// Used by `HttpClientProvider` to configure HTTP requests.
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
            max_idle_per_host: 10,
            idle_timeout: Duration::from_secs(90),
            keepalive: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            user_agent: "mcb/domain-client".to_string(),
        }
    }
}

impl HttpClientConfig {
    /// Create configuration with custom timeout only
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }
}
