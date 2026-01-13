use crate::infrastructure::rate_limit::RateLimitConfig;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Metrics API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MetricsConfig {
    /// Port for metrics HTTP API
    #[validate(range(min = 1))]
    pub port: u16,
    /// Enable metrics collection
    pub enabled: bool,
    /// Rate limiting configuration
    #[validate(nested)]
    pub rate_limiting: RateLimitConfig,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        panic!("[FATAL] MetricsConfig::default() should never be called - all config must come from config/default.toml embedded in binary")
    }
}
