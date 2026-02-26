//! Configuration Tests
//!
//! Tests for configuration loading, validation, and management.

mod config_repro_tests;
mod strict_config_tests;

mod server_tests;
mod types_tests;
mod validation_tests;

use mcb_infrastructure::config::{
    ServerConfig, ServerCorsConfig, ServerNetworkConfig, ServerSslConfig, ServerTimeoutConfig,
    TransportMode,
};

pub(super) fn default_server_config() -> ServerConfig {
    ServerConfig {
        transport_mode: TransportMode::Stdio,
        network: ServerNetworkConfig {
            host: "127.0.0.1".to_owned(),
            port: 8080,
        },
        ssl: ServerSslConfig {
            https: false,
            ssl_cert_path: None,
            ssl_key_path: None,
        },
        timeouts: ServerTimeoutConfig {
            request_timeout_secs: 30,
            connection_timeout_secs: 10,
            max_request_body_size: 10 * 1024 * 1024,
        },
        cors: ServerCorsConfig {
            cors_enabled: true,
            cors_origins: vec!["*".to_owned()],
        },
    }
}

pub(super) fn development_config() -> ServerConfig {
    let mut config = default_server_config();
    config.network.host = "127.0.0.1".to_owned();
    config.network.port = 8080;
    config.ssl.https = false;
    config.timeouts.request_timeout_secs = 60;
    config.cors.cors_enabled = true;
    config.cors.cors_origins = vec!["http://localhost:3000".to_owned(), "*".to_owned()];
    config
}

pub(super) fn testing_config() -> ServerConfig {
    let mut config = default_server_config();
    config.network.host = "127.0.0.1".to_owned();
    config.network.port = 0;
    config.ssl.https = false;
    config.timeouts.request_timeout_secs = 5;
    config.timeouts.connection_timeout_secs = 2;
    config.cors.cors_enabled = false;
    config.cors.cors_origins = vec![];
    config
}

pub(super) fn production_config() -> ServerConfig {
    let mut config = default_server_config();
    config.network.host = "0.0.0.0".to_owned();
    config.network.port = 8443;
    config.ssl.https = true;
    config.timeouts.request_timeout_secs = 30;
    config.timeouts.connection_timeout_secs = 5;
    config.cors.cors_enabled = true;
    config.cors.cors_origins = vec!["https://yourdomain.com".to_owned()];
    config
}
