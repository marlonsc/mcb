use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::config::TransportMode;
use mcb_server::transport::config::TransportConfig;

#[test]
fn test_default_is_stdio() {
    let config = TransportConfig::default();
    let loaded = ConfigLoader::new().load().expect("load config");
    assert_eq!(config.mode, loaded.server.transport_mode);
    assert_eq!(config.http_port, Some(loaded.server.network.port));
    assert_eq!(
        config.http_host.as_deref(),
        Some(loaded.server.network.host.as_str())
    );
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
    let loaded = ConfigLoader::new().load().expect("load config");
    let override_port = loaded.server.network.port.saturating_add(8);
    let config = TransportConfig::http(override_port);
    assert!(matches!(config.mode, TransportMode::Http));
    assert_eq!(config.http_port, Some(override_port));
    assert_eq!(
        config.http_host.as_deref(),
        Some(loaded.server.network.host.as_str())
    );
}

#[test]
fn test_hybrid_constructor() {
    let loaded = ConfigLoader::new().load().expect("load config");
    let override_port = loaded.server.network.port.saturating_add(3);
    let config = TransportConfig::hybrid(override_port);
    assert!(matches!(config.mode, TransportMode::Hybrid));
    assert_eq!(config.http_port, Some(override_port));
    assert_eq!(
        config.http_host.as_deref(),
        Some(loaded.server.network.host.as_str())
    );
}

#[test]
fn test_config_clone() {
    let loaded = ConfigLoader::new().load().expect("load config");
    let override_port = loaded.server.network.port.saturating_add(11);
    let config = TransportConfig::http(override_port);
    let cloned = config.clone();
    assert_eq!(cloned.http_port, Some(override_port));
}

#[test]
fn test_config_debug() {
    let config = TransportConfig::stdio();
    let debug = format!("{:?}", config);
    assert!(debug.contains("TransportConfig"));
}
