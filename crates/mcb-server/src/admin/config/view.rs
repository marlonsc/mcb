//! Sanitized configuration views

use std::collections::HashMap;

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::config::types::{CacheSystemConfig, LimitsConfig, MetricsConfig};
use serde::{Deserialize, Serialize};

/// Sanitized configuration for API responses
///
/// Contains only non-sensitive configuration values suitable for
/// display in admin interfaces.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SanitizedConfig {
    /// Server configuration section
    pub server: ServerConfigView,
    /// Embedding provider configurations
    pub embedding: HashMap<String, EmbeddingConfigView>,
    /// Vector store configurations
    pub vector_store: HashMap<String, VectorStoreConfigView>,
    /// Logging configuration
    pub logging: LoggingConfigView,
    /// Cache configuration
    pub cache: CacheConfigView,
    /// Metrics configuration
    pub metrics: MetricsConfigView,
    /// Limits configuration
    pub limits: LimitsConfigView,
}

impl SanitizedConfig {
    /// Create a sanitized config from AppConfig, removing sensitive fields
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            server: Self::server_view(config),
            embedding: Self::embedding_views(&config.providers.embedding.configs),
            vector_store: Self::vector_store_views(&config.providers.vector_store.configs),
            logging: Self::logging_view(config),
            cache: Self::cache_view(&config.system.infrastructure.cache),
            metrics: Self::metrics_view(&config.system.infrastructure.metrics),
            limits: Self::limits_view(&config.system.infrastructure.limits),
        }
    }

    fn server_view(config: &AppConfig) -> ServerConfigView {
        ServerConfigView {
            host: config.server.network.host.clone(),
            port: config.server.network.port,
            transport_mode: format!("{:?}", config.server.transport_mode),
            https: config.server.ssl.https,
        }
    }

    fn embedding_views(
        cfg: &HashMap<String, EmbeddingConfig>,
    ) -> HashMap<String, EmbeddingConfigView> {
        cfg.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    EmbeddingConfigView {
                        provider: format!("{:?}", v.provider),
                        model: v.model.clone(),
                        dimensions: v.dimensions,
                        has_api_key: v.api_key.is_some(),
                    },
                )
            })
            .collect()
    }

    fn vector_store_views(
        cfg: &HashMap<String, VectorStoreConfig>,
    ) -> HashMap<String, VectorStoreConfigView> {
        cfg.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    VectorStoreConfigView {
                        provider: format!("{:?}", v.provider),
                        dimensions: v.dimensions,
                        collection: v.collection.clone(),
                        has_address: v.address.is_some(),
                    },
                )
            })
            .collect()
    }

    fn logging_view(config: &AppConfig) -> LoggingConfigView {
        LoggingConfigView {
            level: config.logging.level.clone(),
            json_format: config.logging.json_format,
            file_output: config
                .logging
                .file_output
                .as_ref()
                .map(|p: &std::path::PathBuf| p.display().to_string()),
        }
    }

    fn cache_view(c: &CacheSystemConfig) -> CacheConfigView {
        CacheConfigView {
            enabled: c.enabled,
            provider: format!("{:?}", c.provider),
            default_ttl_secs: c.default_ttl_secs,
            max_size: c.max_size,
        }
    }

    fn metrics_view(m: &MetricsConfig) -> MetricsConfigView {
        MetricsConfigView {
            enabled: m.enabled,
            collection_interval_secs: m.collection_interval_secs,
        }
    }

    fn limits_view(l: &LimitsConfig) -> LimitsConfigView {
        LimitsConfigView {
            memory_limit: l.memory_limit,
            cpu_limit: l.cpu_limit,
            max_connections: l.max_connections,
        }
    }
}

/// Server configuration view (non-sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfigView {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Transport mode (Stdio, Http, Hybrid)
    pub transport_mode: String,
    /// HTTPS enabled
    pub https: bool,
}

/// Embedding provider configuration view (non-sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfigView {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Embedding dimensions (if configured)
    pub dimensions: Option<usize>,
    /// Whether an API key is configured (not the key itself)
    pub has_api_key: bool,
}

/// Vector store configuration view (non-sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfigView {
    /// Provider name
    pub provider: String,
    /// Vector dimensions (if configured)
    pub dimensions: Option<usize>,
    /// Collection name
    pub collection: Option<String>,
    /// Whether address is configured (for remote providers)
    pub has_address: bool,
}

/// Logging configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfigView {
    /// Log level
    pub level: String,
    /// JSON format enabled
    pub json_format: bool,
    /// File output path (if configured)
    pub file_output: Option<String>,
}

/// Cache configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfigView {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache provider name
    pub provider: String,
    /// Default TTL in seconds
    pub default_ttl_secs: u64,
    /// Maximum cache size
    pub max_size: usize,
}

/// Metrics configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsConfigView {
    /// Whether metrics are enabled
    pub enabled: bool,
    /// Collection interval in seconds
    pub collection_interval_secs: u64,
}

/// Limits configuration view
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitsConfigView {
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// CPU limit (number of cores)
    pub cpu_limit: usize,
    /// Maximum concurrent connections
    pub max_connections: u32,
}
