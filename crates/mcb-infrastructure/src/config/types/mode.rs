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
    /// Requires server_url to be configured
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeConfig {
    /// Operating mode type
    #[serde(rename = "type")]
    pub mode_type: OperatingMode,

    /// Server URL for client mode
    /// Only used when mode_type = Client
    pub server_url: String,

    /// Session prefix for context isolation
    /// Optional: if set, collections will be prefixed with this value
    pub session_prefix: Option<String>,

    /// Connection timeout in seconds for client mode
    pub timeout_secs: u64,

    /// Enable automatic reconnection on connection loss
    pub auto_reconnect: bool,

    /// Maximum reconnection attempts (0 = unlimited)
    pub max_reconnect_attempts: u32,
}

impl ModeConfig {
    /// Check if running in client mode
    pub fn is_client(&self) -> bool {
        self.mode_type == OperatingMode::Client
    }

    /// Check if running in standalone mode
    pub fn is_standalone(&self) -> bool {
        self.mode_type == OperatingMode::Standalone
    }

    /// Get the server URL (only meaningful in client mode)
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Get session prefix if configured
    pub fn session_prefix(&self) -> Option<&str> {
        self.session_prefix.as_deref()
    }
}
