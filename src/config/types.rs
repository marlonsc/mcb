use crate::core::auth::AuthConfig;
use crate::core::cache::CacheConfig;
use crate::core::database::DatabaseConfig;
use crate::core::hybrid_search::HybridSearchConfig;
use crate::core::limits::ResourceLimitsConfig;
use crate::daemon::DaemonConfig;
use crate::sync::SyncConfig;
use serde::{Deserialize, Serialize};

use super::metrics::MetricsConfig;
use super::providers::{EmbeddingProviderConfig, VectorStoreProviderConfig};
use super::server::ServerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: GlobalProviderConfig,
    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Sync configuration
    #[serde(default)]
    pub sync: SyncConfig,
    /// Daemon configuration
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProviderConfig {
    pub embedding: EmbeddingProviderConfig,
    pub vector_store: VectorStoreProviderConfig,
}

/// Legacy provider config (maintained for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub embedding: crate::core::types::EmbeddingConfig,
    pub vector_store: crate::core::types::VectorStoreConfig,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            embedding: crate::core::types::EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                api_key: None,
                base_url: Some("http://localhost:11434".to_string()),
                dimensions: Some(768),
                max_tokens: Some(8192),
            },
            vector_store: crate::core::types::VectorStoreConfig {
                provider: "in-memory".to_string(),
                address: None,
                token: None,
                collection: None,
                dimensions: Some(768),
            },
        }
    }
}

/// Main application configuration
///
/// Central configuration structure containing all settings for the MCP Context Browser.
/// Supports hierarchical configuration with validation and environment variable overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Server configuration (host, port, etc.)
    pub server: ServerConfig,
    /// AI and vector store provider configurations
    pub providers: ProviderConfig,
    /// Metrics and monitoring configuration
    pub metrics: MetricsConfig,
    /// Authentication and authorization settings
    #[serde(default)]
    pub auth: AuthConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Sync coordination configuration
    pub sync: SyncConfig,
    /// Background daemon configuration
    pub daemon: DaemonConfig,

    /// Resource limits configuration
    #[serde(default)]
    pub resource_limits: ResourceLimitsConfig,

    /// Advanced caching configuration
    #[serde(default)]
    pub cache: CacheConfig,
    pub hybrid_search: HybridSearchConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "MCP Context Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            server: ServerConfig::default(),
            providers: ProviderConfig::default(),
            metrics: MetricsConfig::default(),
            auth: AuthConfig::default(),
            database: DatabaseConfig::default(),
            sync: SyncConfig::default(),
            daemon: DaemonConfig::default(),
            resource_limits: ResourceLimitsConfig::default(),
            cache: CacheConfig::default(),
            hybrid_search: HybridSearchConfig::default(),
        }
    }
}

impl Config {
    /// Get metrics port
    pub fn metrics_port(&self) -> u16 {
        self.metrics.port
    }

    /// Check if metrics are enabled
    pub fn metrics_enabled(&self) -> bool {
        self.metrics.enabled
    }

    /// Get server address string
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get metrics server address string
    pub fn metrics_addr(&self) -> String {
        format!("0.0.0.0:{}", self.metrics.port)
    }
}
