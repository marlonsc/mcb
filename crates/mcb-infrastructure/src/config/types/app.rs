//! Main application configuration

use std::collections::HashMap;
use std::path::PathBuf;

use mcb_domain::value_objects::{EmbeddingConfig, ProjectSettings, VectorStoreConfig};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Database provider configuration entry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    /// Database provider name (e.g. "sqlite", "postgres")
    pub provider: String,
    /// Database file path (for file-based providers like `SQLite`)
    pub path: Option<PathBuf>,
}

/// Database configuration container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfigContainer {
    /// Active database provider name
    pub provider: String,
    /// Named database configurations
    pub configs: HashMap<String, DatabaseConfig>,
}

/// Provider configurations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ProvidersConfig {
    /// Database provider configuration
    pub database: DatabaseConfigContainer,
    /// Embedding provider configuration
    pub embedding: EmbeddingConfigContainer,
    /// Vector store provider configuration
    pub vector_store: VectorStoreConfigContainer,
}

/// Default context settings for MCP operations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct McpContextDefaultsConfig {
    /// Git-related context defaults.
    pub git: McpContextGitDefaultsConfig,
}

/// Indexing configuration for file discovery.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct IndexingConfig {
    /// File extensions to include during indexing.
    pub supported_extensions: Vec<String>,
}

/// MCP server feature configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct McpConfig {
    /// Indexing subsystem settings.
    pub indexing: IndexingConfig,
}

/// Git defaults for MCP context resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct McpContextGitDefaultsConfig {
    /// Default branches to consider.
    pub branches: Vec<String>,
    /// Clone depth limit.
    pub depth: usize,
    /// Glob patterns to exclude from context.
    pub ignore_patterns: Vec<String>,
    /// Whether to include git submodules.
    pub include_submodules: bool,
}

/// Infrastructure configurations

/// Data management configurations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl SystemConfig {
    pub fn fallback() -> Self {
        Self {
            infrastructure: InfrastructureConfig::fallback(),
            data: DataConfig::default(),
        }
    }
}

/// Operations and daemon configurations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    /// MCP server feature configuration.
    pub mcp: McpConfig,
    /// MCP context resolution defaults.
    pub mcp_context: McpContextDefaultsConfig,
    /// Project settings loaded from workspace
    #[serde(skip)]
    pub project_settings: Option<ProjectSettings>,
}

impl AppConfig {
    pub fn fallback() -> Self {
        Self {
            mode: ModeConfig::default(),
            server: ServerConfig::fallback(),
            providers: ProvidersConfig::default(),
            logging: LoggingConfig::default(),
            auth: AuthConfig::default(),
            system: SystemConfig::fallback(),
            operations_daemon: OperationsDaemonConfig::default(),
            mcp: McpConfig::default(),
            mcp_context: McpContextDefaultsConfig::default(),
            project_settings: None,
        }
    }
}
