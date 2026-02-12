//! Server configuration types

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::constants::http::*;

/// Transport mode for MCP server
///
/// Defines how the MCP server communicates with clients.
///
/// # Modes
///
/// | Mode | Description | Use Case |
/// | ------ | ------------- | ---------- |
/// | `Stdio` | Standard I/O streams | CLI tools, IDE integrations |
/// | `Http` | HTTP with SSE | Web clients, REST APIs |
/// | `Hybrid` | Both simultaneously | Dual-interface servers |
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// Standard I/O transport (traditional MCP protocol)
    /// Used for CLI tools and IDE integrations (e.g., Claude Code)
    #[default]
    Stdio,
    /// HTTP transport with Server-Sent Events
    /// Used for web clients and REST API access
    Http,
    /// Both Stdio and HTTP simultaneously
    /// Allows serving both CLI and web clients from the same process
    Hybrid,
}

/// Network configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNetworkConfig {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,
}

/// SSL/TLS configuration for server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerSslConfig {
    /// HTTPS enabled
    pub https: bool,

    /// SSL certificate path (if HTTPS enabled)
    pub ssl_cert_path: Option<PathBuf>,

    /// SSL key path (if HTTPS enabled)
    pub ssl_key_path: Option<PathBuf>,
}

/// Timeout configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTimeoutConfig {
    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Maximum request body size in bytes
    pub max_request_body_size: usize,
}

/// CORS configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCorsConfig {
    /// Enable CORS
    pub cors_enabled: bool,

    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    /// Transport mode (stdio, http, hybrid)
    #[serde(default)]
    pub transport_mode: TransportMode,

    /// Network configuration
    pub network: ServerNetworkConfig,

    /// SSL/TLS configuration
    pub ssl: ServerSslConfig,

    /// Timeout configuration
    pub timeouts: ServerTimeoutConfig,

    /// CORS configuration
    pub cors: ServerCorsConfig,
}

// Default implementations for config structs

/// Returns default network configuration with:
/// - Host and port from infrastructure constants
impl Default for ServerNetworkConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_SERVER_HOST.to_string(),
            port: DEFAULT_HTTP_PORT,
        }
    }
}

/// Returns default timeout configuration with:
/// - Request and connection timeouts from infrastructure constants
/// - Max request body size from infrastructure constants
impl Default for ServerTimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout_secs: REQUEST_TIMEOUT_SECS,
            connection_timeout_secs: CONNECTION_TIMEOUT_SECS,
            max_request_body_size: MAX_REQUEST_BODY_SIZE,
        }
    }
}

use mcb_domain::error::{Error, Result};
/// Returns default CORS configuration with:
/// - CORS enabled
/// - Allow all origins (*)
impl Default for ServerCorsConfig {
    fn default() -> Self {
        Self {
            cors_enabled: true,
            cors_origins: vec![DEFAULT_CORS_ORIGIN.to_string()],
        }
    }
}

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

impl ServerConfig {
    /// Parse server address from configuration
    pub fn parse_address(&self) -> Result<SocketAddr> {
        let ip: IpAddr = self
            .network
            .host
            .parse()
            .map_err(|_| Error::Configuration {
                message: format!("Invalid server host: {}", self.network.host),
                source: None,
            })?;

        Ok(SocketAddr::new(ip, self.network.port))
    }

    /// Get the server URL
    pub fn get_base_url(&self) -> String {
        let protocol = if self.ssl.https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.network.host, self.network.port)
    }

    /// Validate SSL configuration
    pub fn validate_ssl(&self) -> Result<()> {
        if !self.ssl.https {
            return Ok(());
        }

        if self.ssl.ssl_cert_path.is_none() {
            return Err(Error::Configuration {
                message: "SSL certificate path is required when HTTPS is enabled".to_string(),
                source: None,
            });
        }

        if self.ssl.ssl_key_path.is_none() {
            return Err(Error::Configuration {
                message: "SSL key path is required when HTTPS is enabled".to_string(),
                source: None,
            });
        }

        // Check if files exist
        if let (Some(cert_path), Some(key_path)) = (&self.ssl.ssl_cert_path, &self.ssl.ssl_key_path)
        {
            if !cert_path.exists() {
                return Err(Error::Configuration {
                    message: format!(
                        "SSL certificate file does not exist: {}",
                        cert_path.display()
                    ),
                    source: None,
                });
            }

            if !key_path.exists() {
                return Err(Error::Configuration {
                    message: format!("SSL key file does not exist: {}", key_path.display()),
                    source: None,
                });
            }
        }

        Ok(())
    }

    /// Get request timeout duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.timeouts.request_timeout_secs)
    }

    /// Get connection timeout duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.timeouts.connection_timeout_secs)
    }
}

/// Server configuration builder
#[derive(Clone)]
pub struct ServerConfigBuilder {
    config: ServerConfig,
}

impl ServerConfigBuilder {
    /// Create a new server config builder with defaults
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    /// Set the server host
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.config.network.host = host.into();
        self
    }

    /// Set the server port
    pub fn port(mut self, port: u16) -> Self {
        self.config.network.port = port;
        self
    }

    /// Enable HTTPS
    pub fn https(mut self, enabled: bool) -> Self {
        self.config.ssl.https = enabled;
        self
    }

    /// Set SSL certificate and key paths
    pub fn ssl_paths<P: Into<PathBuf>>(mut self, cert_path: P, key_path: P) -> Self {
        self.config.ssl.ssl_cert_path = Some(cert_path.into());
        self.config.ssl.ssl_key_path = Some(key_path.into());
        self
    }

    /// Set request timeout in seconds
    pub fn request_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.request_timeout_secs = seconds;
        self
    }

    /// Set connection timeout in seconds
    pub fn connection_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.connection_timeout_secs = seconds;
        self
    }

    /// Set maximum request body size
    pub fn max_request_body_size(mut self, size: usize) -> Self {
        self.config.timeouts.max_request_body_size = size;
        self
    }

    /// Configure CORS
    pub fn cors(mut self, enabled: bool, origins: Vec<String>) -> Self {
        self.config.cors.cors_enabled = enabled;
        self.config.cors.cors_origins = origins;
        self
    }

    /// Build the server configuration
    pub fn build(self) -> ServerConfig {
        self.config
    }
}

/// Creates a `ServerConfigBuilder` with default values.
impl Default for ServerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Server configuration presets
pub struct ServerConfigPresets;

impl ServerConfigPresets {
    /// Development server configuration
    pub fn development() -> ServerConfig {
        ServerConfigBuilder::new()
            .host("127.0.0.1")
            .port(8080)
            .https(false)
            .request_timeout(60)
            .connection_timeout(10)
            .cors(
                true,
                vec!["http://localhost:3000".to_string(), "*".to_string()],
            )
            .build()
    }

    /// Production server configuration
    pub fn production() -> ServerConfig {
        ServerConfigBuilder::new()
            .host("0.0.0.0")
            .port(DEFAULT_HTTPS_PORT)
            .https(true)
            .request_timeout(30)
            .connection_timeout(5)
            .cors(true, vec!["https://yourdomain.com".to_string()])
            .build()
    }

    /// Testing server configuration
    pub fn testing() -> ServerConfig {
        ServerConfigBuilder::new()
            .host("127.0.0.1")
            .port(0) // Use random available port
            .https(false)
            .request_timeout(5)
            .connection_timeout(2)
            .cors(false, vec![])
            .build()
    }
}
