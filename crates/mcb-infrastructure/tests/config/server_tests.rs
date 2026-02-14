//! Server Configuration Tests

use std::net::SocketAddr;

use mcb_infrastructure::config::{ServerConfig, ServerConfigBuilder, ServerConfigPresets};
use mcb_infrastructure::constants::http::DEFAULT_HTTPS_PORT;
use rstest::*;

#[test]
fn test_parse_address() {
    let mut config = ServerConfig::default();
    config.network.host = "127.0.0.1".to_string();
    config.network.port = 8080;

    let address = config.parse_address().unwrap();
    assert_eq!(address, SocketAddr::from(([127, 0, 0, 1], 8080)));
}

#[rstest]
#[case("127.0.0.1", 8080, false, "http://127.0.0.1:8080")]
#[case("example.com", 8443, true, "https://example.com:8443")]
fn server_url(#[case] host: &str, #[case] port: u16, #[case] https: bool, #[case] expected: &str) {
    let mut config = ServerConfig::default();
    config.network.host = host.to_string();
    config.network.port = port;
    config.ssl.https = https;

    assert_eq!(config.get_base_url(), expected);
}

#[test]
fn test_server_config_builder() {
    let config = ServerConfigBuilder::new()
        .host("0.0.0.0")
        .port(9000)
        .https(true)
        .request_timeout(120)
        .cors(true, vec!["https://app.example.com".to_string()])
        .build();

    assert_eq!(config.network.host, "0.0.0.0");
    assert_eq!(config.network.port, 9000);
    assert!(config.ssl.https);
    assert_eq!(config.timeouts.request_timeout_secs, 120);
    assert!(config.cors.cors_enabled);
    assert_eq!(config.cors.cors_origins, vec!["https://app.example.com"]);
}

#[test]
fn test_presets() {
    let dev_config = ServerConfigPresets::development();
    assert_eq!(dev_config.network.host, "127.0.0.1");
    assert_eq!(dev_config.network.port, 8080);
    assert!(!dev_config.ssl.https);

    let prod_config = ServerConfigPresets::production();
    assert_eq!(prod_config.network.host, "0.0.0.0");
    assert_eq!(prod_config.network.port, DEFAULT_HTTPS_PORT);
    assert!(prod_config.ssl.https);

    let test_config = ServerConfigPresets::testing();
    assert_eq!(test_config.network.port, 0); // Random port
    assert!(!test_config.ssl.https);
}
