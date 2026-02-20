//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../docs/modules/infrastructure.md#configuration)
//!
//! Infrastructure configuration types
//!
//! configuration for infrastructure concerns:
//! logging, limits, cache, metrics, and resilience.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ============================================================================
// Logging Configuration
// ============================================================================

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Enable JSON output format
    pub json_format: bool,
    /// Log to file in addition to stdout
    pub file_output: Option<PathBuf>,
    /// Maximum file size before rotation (bytes)
    pub max_file_size: u64,
    /// Maximum number of rotated files to keep
    pub max_files: usize,
    /// Minimum log level forwarded to event bus SSE (trace, debug, info, warn, error)
    pub event_bus_level: String,
}

// ============================================================================
// Resource Limits Configuration
// ============================================================================

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LimitsConfig {
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// CPU limit (number of cores)
    pub cpu_limit: usize,
    /// Disk I/O limit in bytes per second
    pub disk_io_limit: u64,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Maximum concurrent requests per connection
    pub max_requests_per_connection: u32,
}

// ============================================================================
// Cache Configuration
// ============================================================================

/// Cache providers for infrastructure caching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CacheProvider {
    /// In-memory cache (Moka)
    #[default]
    Moka,
    /// Distributed cache (Redis)
    Redis,
}

impl CacheProvider {
    /// Get the provider name as a string for registry lookup
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheProvider::Moka => "moka",
            CacheProvider::Redis => "redis",
        }
    }
}

/// Infrastructure cache system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct CacheSystemConfig {
    /// Cache enabled
    pub enabled: bool,
    /// Cache provider
    pub provider: CacheProvider,
    /// Default TTL in seconds
    pub default_ttl_secs: u64,
    /// Maximum cache size in bytes
    pub max_size: usize,
    /// Redis URL (for Redis provider)
    pub redis_url: Option<String>,
    /// Redis connection pool size
    pub redis_pool_size: u32,
    /// Namespace for cache keys
    pub namespace: String,
}

// ============================================================================
// Metrics Configuration
// ============================================================================

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    /// Metrics enabled
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
    /// Prometheus metrics prefix
    pub prefix: String,
    /// Metrics endpoint enabled
    pub endpoint_enabled: bool,
    /// Metrics endpoint path
    pub endpoint_path: String,
    /// External metrics exporter URL
    pub exporter_url: Option<String>,
}

// ============================================================================
// Resilience Configuration
// ============================================================================

/// Resilience configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ResilienceConfig {
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout_secs: u64,
    /// Circuit breaker success threshold
    pub circuit_breaker_success_threshold: u32,
    /// Rate limiter requests per second
    pub rate_limiter_rps: u32,
    /// Rate limiter burst size
    pub rate_limiter_burst: u32,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}
