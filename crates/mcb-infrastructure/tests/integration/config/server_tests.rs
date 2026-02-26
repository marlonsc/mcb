//! Server Configuration Tests

use rstest::rstest;
use std::net::SocketAddr;

use super::{default_server_config, development_config, production_config, testing_config};

#[test]
fn test_parse_address() {
    let mut config = default_server_config();
    config.network.host = "127.0.0.1".to_owned();
    config.network.port = 8080;

    let address = config.parse_address().unwrap();
    assert_eq!(address, SocketAddr::from(([127, 0, 0, 1], 8080)));
}

#[rstest]
#[case("127.0.0.1", 8080, false, "http://127.0.0.1:8080")]
#[case("example.com", 8443, true, "https://example.com:8443")]
fn server_url(#[case] host: &str, #[case] port: u16, #[case] https: bool, #[case] expected: &str) {
    let mut config = default_server_config();
    config.network.host = host.to_owned();
    config.network.port = port;
    config.ssl.https = https;

    assert_eq!(config.get_base_url(), expected);
}

#[test]
fn test_server_config_builder() {
    let mut config = default_server_config();
    config.network.host = "0.0.0.0".to_owned();
    config.network.port = 9000;
    config.ssl.https = true;
    config.timeouts.request_timeout_secs = 120;
    config.cors.cors_enabled = true;
    config.cors.cors_origins = vec!["https://app.example.com".to_owned()];

    assert_eq!(config.network.host, "0.0.0.0");
    assert_eq!(config.network.port, 9000);
    assert!(config.ssl.https);
    assert_eq!(config.timeouts.request_timeout_secs, 120);
    assert!(config.cors.cors_enabled);
    assert_eq!(config.cors.cors_origins, vec!["https://app.example.com"]);
}

#[test]
fn test_presets() {
    let dev_config = development_config();
    assert_eq!(dev_config.network.host, "127.0.0.1");
    assert_eq!(dev_config.network.port, 8080);
    assert!(!dev_config.ssl.https);

    let prod_config = production_config();
    assert_eq!(prod_config.network.host, "0.0.0.0");
    assert_eq!(prod_config.network.port, 8443);
    assert!(prod_config.ssl.https);

    let test_config = testing_config();
    assert_eq!(test_config.network.port, 0); // Random port
    assert!(!test_config.ssl.https);
}
