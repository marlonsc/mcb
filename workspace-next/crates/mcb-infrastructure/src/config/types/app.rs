//! Main application configuration

use crate::constants::*;
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export all config types
pub use super::{
    auth::{AuthConfig, PasswordAlgorithm},
    backup::BackupConfig,
    cache::{CacheConfig, CacheProvider},
    daemon::DaemonConfig,
    limits::LimitsConfig,
    logging::LoggingConfig,
    metrics::MetricsConfig,
    operations::OperationsConfig,
    resilience::ResilienceConfig,
    server::{ServerConfig, TransportMode},
    snapshot::SnapshotConfig,
    sync::SyncConfig,
};

/// Main application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Embedding provider configurations
    pub embedding: HashMap<String, EmbeddingConfig>,

    /// Vector store provider configurations
    pub vector_store: HashMap<String, VectorStoreConfig>,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Resilience configuration
    pub resilience: ResilienceConfig,

    /// Limits configuration
    pub limits: LimitsConfig,

    /// Daemon configuration
    pub daemon: DaemonConfig,

    /// Backup configuration
    pub backup: BackupConfig,

    /// Snapshot configuration
    pub snapshot: SnapshotConfig,

    /// Sync configuration
    pub sync: SyncConfig,

    /// Operations configuration
    pub operations: OperationsConfig,
}