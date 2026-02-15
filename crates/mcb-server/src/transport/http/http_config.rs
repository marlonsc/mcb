use std::net::SocketAddr;

use mcb_infrastructure::config::ConfigLoader;

/// Configuration for the HTTP transport layer.
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Bind address (e.g. `"127.0.0.1"`).
    pub host: String,
    /// Bind port (e.g. `8080`).
    pub port: u16,
    /// Whether CORS headers are attached to responses.
    pub enable_cors: bool,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        let config = ConfigLoader::new()
            .load()
            .expect("HttpTransportConfig::default requires loadable configuration file");
        Self {
            host: config.server.network.host,
            port: config.server.network.port,
            enable_cors: config.server.cors.cors_enabled,
        }
    }
}

impl HttpTransportConfig {
    /// Build a config bound to the configured host with a custom port.
    pub fn localhost(port: u16) -> Self {
        let config = ConfigLoader::new()
            .load()
            .expect("HttpTransportConfig::localhost requires loadable configuration file");
        Self {
            host: config.server.network.host,
            port,
            enable_cors: config.server.cors.cors_enabled,
        }
    }

    /// Resolve the configured host and port into a [`SocketAddr`].
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid host/port in configuration")
    }
}
