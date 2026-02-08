//! Tests for HTTP transport configuration and types

use mcb_server::transport::http::HttpTransportConfig;

#[test]
fn test_http_config_default() {
    let config = HttpTransportConfig::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert!(config.enable_cors);
}

#[test]
fn test_http_config_localhost() {
    let config = HttpTransportConfig::localhost(3000);
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert!(config.enable_cors);
}

#[test]
fn test_http_config_socket_addr() {
    let config = HttpTransportConfig::localhost(9090);
    let addr = config.socket_addr();
    assert_eq!(addr.port(), 9090);
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
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
