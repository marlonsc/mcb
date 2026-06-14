//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../docs/modules/infrastructure.md#configuration)
//!
//! Operating mode configuration
//!
//! Defines how MCB operates: standalone (local providers), client (connects to server),
//! or server (daemon mode, triggered by --server flag).

use serde::{Deserialize, Serialize};

/// Operating mode for MCB
///
/// Determines how MCB behaves when started without the `--server` flag:
/// - `Standalone`: Run with local providers (default)
/// - `Client`: Connect to a remote MCB server via HTTP
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    /// Standalone mode: run with local providers
    /// This is the default mode
    #[default]
    Standalone,

    /// Client mode: connect to remote MCB server
    /// Requires `server_url` to be configured
    Client,
}

/// Mode configuration section
///
/// Controls how MCB operates:
///
/// ```toml
/// [mode]
/// type = "client"                         # "standalone" or "client"
/// server_url = "http://127.0.0.1:3000"   # Server URL for client mode
/// session_prefix = "claude"               # Optional prefix for session isolation
/// ```
///
/// When `--server` flag is used, mode configuration is ignored and MCB runs as server.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModeConfig {
    /// Operating mode type (`standalone` or `client`).
    #[serde(rename = "type")]
    pub mode_type: OperatingMode,
    /// Server URL for client mode.
    pub server_url: String,
    /// Optional prefix for session isolation.
    pub session_prefix: Option<String>,
    /// Request timeout in seconds (client mode).
    pub timeout_secs: u64,
    /// Whether to auto-reconnect on connection loss (client mode).
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts before giving up.
    pub max_reconnect_attempts: u32,
    /// Active session identifier for session resumption.
    pub session_id: Option<String>,
    /// Path to session state file for persistence.
    pub session_file: Option<String>,
}

impl ModeConfig {
    /// Check if running in client mode
    #[must_use]
    pub fn is_client(&self) -> bool {
        self.mode_type == OperatingMode::Client
    }

    /// Check if running in standalone mode
    #[must_use]
    pub fn is_standalone(&self) -> bool {
        self.mode_type == OperatingMode::Standalone
    }

    /// Get the server URL (only meaningful in client mode)
    #[must_use]
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Get session prefix if configured
    #[must_use]
    pub fn session_prefix(&self) -> Option<&str> {
        self.session_prefix.as_deref()
    }
}
