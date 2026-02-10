//! Tests for HTTP transport configuration and types

use mcb_infrastructure::config::ConfigLoader;
use mcb_server::transport::http::HttpTransportConfig;

#[test]
fn test_http_config_default() {
    let config = HttpTransportConfig::default();
    let loaded = ConfigLoader::new().load().expect("load config");
    assert_eq!(config.host, loaded.server.network.host);
    assert_eq!(config.port, loaded.server.network.port);
    assert!(config.enable_cors);
}

#[test]
fn test_http_config_localhost() {
    let loaded = ConfigLoader::new().load().expect("load config");
    let override_port = loaded.server.network.port.saturating_add(7);
    let config = HttpTransportConfig::localhost(override_port);
    assert_eq!(config.host, loaded.server.network.host);
    assert_eq!(config.port, override_port);
    assert!(config.enable_cors);
}

#[test]
fn test_http_config_socket_addr() {
    let loaded = ConfigLoader::new().load().expect("load config");
    let override_port = loaded.server.network.port.saturating_add(9);
    let config = HttpTransportConfig::localhost(override_port);
    let addr = config.socket_addr();
    assert_eq!(addr.port(), override_port);
    assert_eq!(addr.ip().to_string(), loaded.server.network.host);
}

#[test]
fn test_http_config_clone() {
    let config = HttpTransportConfig {
        host: "0.0.0.0".to_string(),
        port: 4000,
        enable_cors: false,
    };
    let cloned = config.clone();
    assert_eq!(cloned.host, "0.0.0.0");
    assert_eq!(cloned.port, 4000);
    assert!(!cloned.enable_cors);
}

#[test]
fn test_http_config_debug() {
    let config = HttpTransportConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("HttpTransportConfig"));
}
