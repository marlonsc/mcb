//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../docs/modules/infrastructure.md#configuration)
//!
//! Configuration types module
//!
//! configuration types organized by domain:
//! - `app` - Main application configuration and containers
//! - `server` - Server transport and network configuration
//! - `mode` - Operating mode configuration
//! - `infrastructure` - Logging, limits, cache, metrics, resilience
//! - `system` - Auth, `event_bus`, backup, sync, snapshot, daemon, operations

pub mod app;
pub mod infrastructure;
pub mod mode;
pub mod server;
pub mod system;

// Re-export main types from app (which already re-exports from sub-modules)
pub use app::{
    AdminApiKeyConfig, ApiKeyConfig, AppConfig, AuthConfig, BackupConfig, CacheProvider,
    CacheSystemConfig, DaemonConfig, DataConfig, DatabaseConfig, DatabaseConfigContainer,
    EmbeddingConfigContainer, EventBusBackend, EventBusConfig, IndexingConfig,
    InfrastructureConfig, JwtConfig, LimitsConfig, LoggingConfig, McpConfig, ModeConfig,
    OperatingMode, OperationsConfig, OperationsDaemonConfig, PasswordAlgorithm, ProvidersConfig,
    ResilienceConfig, ServerConfig, ServerConfigBuilder, ServerConfigPresets, ServerCorsConfig,
    ServerNetworkConfig, ServerSslConfig, ServerTimeoutConfig, SnapshotConfig, SyncConfig,
    SystemConfig, TransportMode, VectorStoreConfigContainer,
};
// Also re-export MetricsConfig which is only in infrastructure
pub use infrastructure::MetricsConfig;
