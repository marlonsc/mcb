use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::config::TransportMode;
use mcb_server::transport::config::TransportConfig;
use rstest::rstest;

#[test]
fn test_from_server_config() {
    let loaded = ConfigLoader::new().load().expect("load config");
    let config = TransportConfig::from_server_config(&loaded.server);
    assert_eq!(config.mode, loaded.server.transport_mode);
    assert_eq!(config.http_port, Some(loaded.server.network.port));
    assert_eq!(
        config.http_host.as_deref(),
        Some(loaded.server.network.host.as_str())
    );
}

#[rstest]
#[case("stdio", 0, TransportMode::Stdio)]
#[case("http", 8, TransportMode::Http)]
#[case("hybrid", 3, TransportMode::Hybrid)]
fn test_transport_constructors(
    #[case] kind: &str,
    #[case] offset: u16,
    #[case] expected_mode: TransportMode,
) {
    let loaded = ConfigLoader::new().load().expect("load config");
    let override_port = loaded.server.network.port.saturating_add(offset);
    let host = loaded.server.network.host.clone();
    let config = match kind {
        "stdio" => TransportConfig::stdio(),
        "http" => TransportConfig::http(host, override_port),
        "hybrid" => TransportConfig::hybrid(host, override_port),
        _ => panic!("unknown constructor kind"),
    };

    assert_eq!(config.mode, expected_mode);
    if expected_mode == TransportMode::Stdio {
        assert!(config.http_port.is_none());
        assert!(config.http_host.is_none());
        return;
    }
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
    let config = TransportConfig::http(loaded.server.network.host.clone(), override_port);
    let cloned = config.clone();
    assert_eq!(cloned.http_port, Some(override_port));
}

#[test]
fn test_config_debug() {
    let config = TransportConfig::stdio();
    let debug = format!("{config:?}");
    assert!(debug.contains("TransportConfig"));
}
