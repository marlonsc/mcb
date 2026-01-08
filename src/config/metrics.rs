use crate::core::rate_limit::RateLimitConfig;
use serde::{Deserialize, Serialize};

/// Metrics API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Port for metrics HTTP API
    pub port: u16,
    /// Enable metrics collection
    pub enabled: bool,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limiting: RateLimitConfig,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            port: 3001,
            enabled: true,
            rate_limiting: RateLimitConfig::default(),
        }
    }
}
