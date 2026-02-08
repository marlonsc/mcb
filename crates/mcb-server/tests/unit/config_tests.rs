use mcb_infrastructure::config::TransportMode;
use mcb_server::transport::config::TransportConfig;

#[test]
fn test_default_is_stdio() {
    let config = TransportConfig::default();
    assert!(matches!(config.mode, TransportMode::Stdio));
    assert!(config.http_port.is_none());
    assert!(config.http_host.is_none());
}

#[test]
fn test_stdio_constructor() {
    let config = TransportConfig::stdio();
    assert!(matches!(config.mode, TransportMode::Stdio));
    assert!(config.http_port.is_none());
    assert!(config.http_host.is_none());
}

#[test]
fn test_http_constructor() {
    let config = TransportConfig::http(8080);
    assert!(matches!(config.mode, TransportMode::Http));
    assert_eq!(config.http_port, Some(8080));
    assert_eq!(config.http_host.as_deref(), Some("127.0.0.1"));
}

#[test]
fn test_hybrid_constructor() {
    let config = TransportConfig::hybrid(3000);
    assert!(matches!(config.mode, TransportMode::Hybrid));
    assert_eq!(config.http_port, Some(3000));
    assert_eq!(config.http_host.as_deref(), Some("127.0.0.1"));
}

#[test]
fn test_config_clone() {
    let config = TransportConfig::http(9090);
    let cloned = config.clone();
    assert_eq!(cloned.http_port, Some(9090));
}

#[test]
fn test_config_debug() {
    let config = TransportConfig::stdio();
    let debug = format!("{:?}", config);
    assert!(debug.contains("TransportConfig"));
}
