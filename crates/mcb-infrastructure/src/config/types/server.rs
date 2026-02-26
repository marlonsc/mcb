//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../docs/modules/infrastructure.md#configuration)
//!
//! Server configuration types

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Network configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerNetworkConfig {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,
}

/// SSL/TLS configuration for server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ServerCorsConfig {
    /// Enable CORS
    pub cors_enabled: bool,

    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    /// Network configuration
    pub network: ServerNetworkConfig,

    /// SSL/TLS configuration
    pub ssl: ServerSslConfig,

    /// Timeout configuration
    pub timeouts: ServerTimeoutConfig,

    /// CORS configuration
    pub cors: ServerCorsConfig,
}

use mcb_domain::error::{Error, Result};

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

impl ServerConfig {
    /// Parse server address from configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the host address cannot be parsed.
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
    #[must_use]
    pub fn get_base_url(&self) -> String {
        let protocol = if self.ssl.https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.network.host, self.network.port)
    }

    /// Validate SSL configuration
    ///
    /// # Errors
    ///
    /// Returns an error if HTTPS is enabled but certificate or key paths are missing.
    pub fn validate_ssl(&self) -> Result<()> {
        if !self.ssl.https {
            return Ok(());
        }

        if self.ssl.ssl_cert_path.is_none() {
            return Err(Error::Configuration {
                message: "SSL certificate path is required when HTTPS is enabled".to_owned(),
                source: None,
            });
        }

        if self.ssl.ssl_key_path.is_none() {
            return Err(Error::Configuration {
                message: "SSL key path is required when HTTPS is enabled".to_owned(),
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
    #[must_use]
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.timeouts.request_timeout_secs)
    }

    /// Get connection timeout duration
    #[must_use]
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.timeouts.connection_timeout_secs)
    }
}
