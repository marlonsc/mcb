//! Main application configuration

use std::collections::HashMap;
use std::path::PathBuf;

use mcb_domain::value_objects::{EmbeddingConfig, ProjectSettings, VectorStoreConfig};
use serde::{Deserialize, Serialize};

// Re-export all config types from modules
pub use super::infrastructure::{
    CacheProvider, CacheSystemConfig, LimitsConfig, LoggingConfig, MetricsConfig, ResilienceConfig,
};
pub use super::mode::{ModeConfig, OperatingMode};
pub use super::server::{
    ServerConfig, ServerConfigBuilder, ServerConfigPresets, ServerCorsConfig, ServerNetworkConfig,
    ServerSslConfig, ServerTimeoutConfig, TransportMode,
};
pub use super::system::{
    AdminApiKeyConfig, ApiKeyConfig, AuthConfig, BackupConfig, DaemonConfig, EventBusConfig,
    EventBusProvider, JwtConfig, OperationsConfig, PasswordAlgorithm, SnapshotConfig, SyncConfig,
};

/// Embedding configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingConfigContainer {
    /// Provider name
    pub provider: Option<String>,
    /// Model name
    pub model: Option<String>,
    /// Base URL for API
    pub base_url: Option<String>,
    /// API key
    pub api_key: Option<String>,
    /// Embedding dimensions
    pub dimensions: Option<usize>,
    /// Cache directory for local embedding providers
    pub cache_dir: Option<PathBuf>,
    /// Named configs for TOML format
    pub configs: HashMap<String, EmbeddingConfig>,
}

/// Vector store configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorStoreConfigContainer {
    /// Provider name
    pub provider: Option<String>,
    /// Server address
    pub address: Option<String>,
    /// Embedding dimensions
    pub dimensions: Option<usize>,
    /// Collection name
    pub collection: Option<String>,
    /// Named configs for TOML format
    pub configs: HashMap<String, VectorStoreConfig>,
}

/// Provider configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvidersConfig {
    /// Database provider name (e.g. "sqlite", "postgres")
    pub database: String,
    /// Embedding provider configuration
    pub embedding: EmbeddingConfigContainer,
    /// Vector store provider configuration
    pub vector_store: VectorStoreConfigContainer,
}

/// Infrastructure configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfrastructureConfig {
    /// Cache system configuration
    pub cache: CacheSystemConfig,
    /// EventBus configuration
    pub event_bus: EventBusConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Resilience configuration
    pub resilience: ResilienceConfig,
    /// Limits configuration
    pub limits: LimitsConfig,
}

/// Data management configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataConfig {
    /// Snapshot configuration
    pub snapshot: SnapshotConfig,
    /// Sync configuration
    pub sync: SyncConfig,
    /// Backup configuration
    pub backup: BackupConfig,
}

/// System infrastructure and data configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemConfig {
    /// Infrastructure configurations
    pub infrastructure: InfrastructureConfig,
    /// Data management configurations
    pub data: DataConfig,
}

/// Operations and daemon configurations combined
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationsDaemonConfig {
    /// Daemon configuration
    pub daemon: DaemonConfig,
    /// Operations configuration
    pub operations: OperationsConfig,
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// Operating mode configuration
    pub mode: ModeConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: ProvidersConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// System configurations
    pub system: SystemConfig,
    /// Operations and daemon configurations
    pub operations_daemon: OperationsDaemonConfig,
    /// Project settings loaded from workspace
    #[serde(skip)]
    pub project_settings: Option<ProjectSettings>,
}

impl Default for AppConfig {
    fn default() -> Self {
        toml::from_str(include_str!("../../../../../config/default.toml"))
            .expect("AppConfig::default requires valid config/default.toml")
    }
}
