//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../docs/modules/infrastructure.md#configuration)
//!
//! System configuration types
//!
//! configuration for system concerns:
//! auth, `event_bus`, backup, sync, snapshot, daemon, and operations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use mcb_providers::constants::EVENT_BUS_DEFAULT_CAPACITY;

use crate::constants::events::{EVENT_BUS_CONNECTION_TIMEOUT_MS, EVENT_BUS_MAX_RECONNECT_ATTEMPTS};

// ============================================================================
// Authentication Configuration
// ============================================================================

/// Password hashing algorithms for authentication.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum PasswordAlgorithm {
    /// Argon2 password hashing algorithm.
    #[default]
    Argon2,
    /// Bcrypt password hashing algorithm.
    Bcrypt,
    /// PBKDF2 password hashing algorithm.
    Pbkdf2,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct JwtConfig {
    /// JWT secret key (REQUIRED when auth enabled, min 32 chars)
    pub secret: String,
    /// JWT expiration time in seconds
    pub expiration_secs: u64,
    /// JWT refresh token expiration in seconds
    pub refresh_expiration_secs: u64,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ApiKeyConfig {
    /// API key authentication enabled
    pub enabled: bool,
    /// API key header name
    pub header: String,
}

/// Admin API key configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AdminApiKeyConfig {
    /// Admin API key authentication enabled
    pub enabled: bool,
    /// Header name for admin API key
    pub header: String,
    /// The actual admin API key
    pub key: Option<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// JWT configuration
    pub jwt: JwtConfig,
    /// API key configuration
    pub api_key: ApiKeyConfig,
    /// Admin API key configuration
    pub admin: AdminApiKeyConfig,
    /// Password hashing algorithm
    pub password_algorithm: PasswordAlgorithm,
}

// ============================================================================
// EventBus Configuration
// ============================================================================

/// Event bus backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EventBusBackend {
    /// In-process broadcast channel.
    #[default]
    #[serde(alias = "tokio")]
    InProcess,
}

/// Event bus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventBusConfig {
    /// Event bus backend to use.
    pub provider: EventBusBackend,
    /// Buffer capacity for the event bus.
    pub capacity: usize,
    /// Connection timeout in milliseconds.
    pub connection_timeout_ms: u64,
    /// Maximum reconnection attempts.
    pub max_reconnect_attempts: u32,
}

impl EventBusConfig {
    /// Creates default in-process event bus configuration.
    #[must_use]
    pub fn in_process() -> Self {
        Self {
            provider: EventBusBackend::InProcess,
            capacity: EVENT_BUS_DEFAULT_CAPACITY,
            connection_timeout_ms: EVENT_BUS_CONNECTION_TIMEOUT_MS,
            max_reconnect_attempts: EVENT_BUS_MAX_RECONNECT_ATTEMPTS,
        }
    }

    /// Creates in-process event bus configuration with custom buffer capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            provider: EventBusBackend::InProcess,
            capacity,
            connection_timeout_ms: EVENT_BUS_CONNECTION_TIMEOUT_MS,
            max_reconnect_attempts: EVENT_BUS_MAX_RECONNECT_ATTEMPTS,
        }
    }

    /// Default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::in_process()
    }

    /// Returns the fallback event bus configuration.
    #[must_use]
    pub fn fallback() -> Self {
        Self::in_process()
    }
}

// ============================================================================
// Backup Configuration
// ============================================================================

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BackupConfig {
    /// Backup enabled
    pub enabled: bool,
    /// Backup directory
    pub directory: PathBuf,
    /// Backup interval in seconds
    pub interval_secs: u64,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Compress backups
    pub compress: bool,
    /// Encrypt backups
    pub encrypt: bool,
    /// Backup encryption key (if encryption enabled)
    pub encryption_key: Option<String>,
}

// ============================================================================
// Sync Configuration
// ============================================================================

/// System-level runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemConfig {
    /// Event bus settings.
    pub events: EventBusConfig,
}

impl SystemConfig {
    /// Returns the fallback system runtime configuration.
    #[must_use]
    pub fn fallback() -> Self {
        Self {
            events: EventBusConfig::fallback(),
        }
    }
}
/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {
    /// Sync enabled
    pub enabled: bool,
    /// Enable file watching for hot-reload
    pub watching_enabled: bool,
    /// Sync batch size
    pub batch_size: usize,
    /// Sync debounce delay in milliseconds
    pub debounce_delay_ms: u64,
    /// Sync timeout in seconds
    pub timeout_secs: u64,
    /// Maximum concurrent sync operations
    pub max_concurrent: usize,
}

// ============================================================================
// Snapshot Configuration
// ============================================================================

/// Snapshot configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SnapshotConfig {
    /// Snapshot enabled
    pub enabled: bool,
    /// Snapshot directory
    pub directory: PathBuf,
    /// Maximum file size for snapshot operations
    pub max_file_size: usize,
    /// Snapshot compression enabled
    pub compression_enabled: bool,
    /// Change detection enabled
    pub change_detection_enabled: bool,
}

// ============================================================================
// Daemon Configuration
// ============================================================================

/// Daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DaemonConfig {
    /// Daemon enabled
    pub enabled: bool,
    /// Process check interval in seconds
    pub check_interval_secs: u64,
    /// Restart delay in seconds
    pub restart_delay_secs: u64,
    /// Maximum restart attempts
    pub max_restart_attempts: u32,
    /// Auto-start daemon
    pub auto_start: bool,
}

// ============================================================================
// Operations Configuration
// ============================================================================

/// Operations configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct OperationsConfig {
    /// Operations tracking enabled
    pub tracking_enabled: bool,
    /// Operations cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Operations retention period in seconds
    pub retention_secs: u64,
    /// Maximum operations to keep in memory
    pub max_operations_in_memory: usize,
}
