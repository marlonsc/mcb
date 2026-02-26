//! Configuration Validation Tests
//!
//! Tests for configuration validation across all config types.

use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

use mcb_infrastructure::config::{
    CacheProvider, CacheSystemConfig, ServerConfig, ServerSslConfig, TestConfigBuilder,
};

use super::{default_server_config, development_config, production_config, testing_config};

fn loaded_config() -> mcb_infrastructure::config::AppConfig {
    TestConfigBuilder::new()
        .and_then(|b| b.build().map(|(config, _)| config))
        .expect("load config")
}

#[rstest]
#[case(0)]
#[case(80)]
#[case(443)]
#[case(65535)]
fn server_config_port_validation(#[case] port: u16) {
    let mut config = default_server_config();
    config.network.port = port;
    assert_eq!(config.network.port, port);
}

#[test]
fn server_config_address_parsing() {
    let mut config = default_server_config();
    config.network.host = "127.0.0.1".to_owned();
    config.network.port = 8080;
    let addr = config.parse_address().unwrap();
    assert_eq!(addr.port(), 8080);
}

#[test]
#[serial]
fn test_auth_config_jwt_secret_length() {
    let default_auth = loaded_config().auth;
    assert!(
        !default_auth.jwt.secret.is_empty(),
        "JWT secret must be configured in test YAML"
    );
    assert!(
        default_auth.jwt.secret.len() >= 32,
        "Configured JWT secret should be at least 32 characters"
    );

    // Custom secret can be set - minimum 32 characters required
    let mut custom_auth = default_auth.clone();
    custom_auth.jwt.secret = "custom_secret_at_least_32_chars_long!".to_owned();
    assert_eq!(custom_auth.jwt.secret.len(), 37);
    assert!(
        custom_auth.jwt.secret.len() >= 32,
        "Configured JWT secret should be at least 32 characters"
    );

    // Expiration times should be reasonable
    assert!(default_auth.jwt.expiration_secs > 0);
    assert!(default_auth.jwt.refresh_expiration_secs > default_auth.jwt.expiration_secs);
}

#[test]
#[serial]
fn test_cache_config_ttl_when_enabled() {
    // When cache is enabled, TTL should be positive
    let enabled_cache = CacheSystemConfig {
        enabled: true,
        provider: CacheProvider::Moka,
        default_ttl_secs: 300,
        max_size: 1024 * 1024,
        redis_url: None,
        redis_pool_size: 8,
        namespace: "test".to_owned(),
    };
    assert!(enabled_cache.default_ttl_secs > 0);
    assert!(enabled_cache.max_size > 0);

    // Default cache config has reasonable TTL
    let default_cache = loaded_config().system.infrastructure.cache;
    assert!(
        default_cache.default_ttl_secs >= 60,
        "Default TTL should be at least 60 seconds"
    );
    assert!(
        default_cache.max_size >= 1024,
        "Default max size should be at least 1KB"
    );

    // Disabled cache still maintains valid config
    let disabled_cache = CacheSystemConfig {
        enabled: false,
        provider: default_cache.provider.clone(),
        default_ttl_secs: default_cache.default_ttl_secs,
        max_size: default_cache.max_size,
        redis_url: default_cache.redis_url.clone(),
        redis_pool_size: default_cache.redis_pool_size,
        namespace: default_cache.namespace.clone(),
    };
    assert!(!disabled_cache.enabled);
    // TTL and size should still be valid even when disabled
    assert!(disabled_cache.default_ttl_secs > 0);

    // Redis provider config (dummy URL for unit validation only)
    let redis_cache = CacheSystemConfig {
        enabled: true,
        provider: CacheProvider::Redis,
        default_ttl_secs: default_cache.default_ttl_secs,
        max_size: default_cache.max_size,
        redis_url: Some("redis://127.0.0.1:6379".to_owned()),
        redis_pool_size: 16,
        namespace: default_cache.namespace.clone(),
    };
    assert!(redis_cache.redis_url.is_some());
    assert!(redis_cache.redis_pool_size > 0);
}

#[test]
fn test_ssl_cert_required_for_https() {
    // HTTPS without SSL paths should fail validation
    let https_no_ssl = ServerConfig {
        network: default_server_config().network,
        ssl: ServerSslConfig {
            https: true,
            ssl_cert_path: None,
            ssl_key_path: None,
        },
        timeouts: default_server_config().timeouts,
        cors: default_server_config().cors,
    };
    let result = https_no_ssl.validate_ssl();
    let err = result.expect_err("HTTPS without SSL paths should fail validation");
    assert!(
        err.to_string().contains("certificate path is required"),
        "error should mention certificate path: {err}"
    );

    // HTTPS with only cert path should fail
    let https_cert_only = ServerConfig {
        network: default_server_config().network,
        ssl: ServerSslConfig {
            https: true,
            ssl_cert_path: Some(PathBuf::from("/path/to/cert.pem")),
            ssl_key_path: None,
        },
        timeouts: default_server_config().timeouts,
        cors: default_server_config().cors,
    };
    let result = https_cert_only.validate_ssl();
    let err = result.expect_err("HTTPS with cert only should fail validation");
    assert!(
        err.to_string().contains("key path is required"),
        "error should mention key path: {err}"
    );

    // HTTP config doesn't require SSL
    let http_config = ServerConfig {
        network: default_server_config().network,
        ssl: ServerSslConfig {
            https: false,
            ssl_cert_path: None,
            ssl_key_path: None,
        },
        timeouts: default_server_config().timeouts,
        cors: default_server_config().cors,
    };
    let result = http_config.validate_ssl();
    result.expect("HTTP config should pass SSL validation");
}

#[test]
#[serial]
fn test_default_config_is_valid() {
    // Default server config should be parseable
    let server_config = default_server_config();
    let addr_result = server_config.parse_address();
    assert!(
        addr_result.is_ok(),
        "Default server config should have valid address"
    );

    // Default SSL config (HTTP) should be valid
    let ssl_result = server_config.validate_ssl();
    assert!(
        ssl_result.is_ok(),
        "Default server config (HTTP) should pass SSL validation"
    );

    // Presets should produce valid configs
    let dev_config = development_config();
    assert!(dev_config.parse_address().is_ok());
    assert!(dev_config.validate_ssl().is_ok());

    let test_config = testing_config();
    assert!(test_config.parse_address().is_ok());
    assert!(test_config.validate_ssl().is_ok());

    // Production preset has HTTPS but no SSL paths - that's expected for "template"
    // Users must provide real SSL paths for production
    let prod_config = production_config();
    assert!(prod_config.parse_address().is_ok());
    // Production config is a template, SSL paths must be added by user
    // So we just verify the address is valid

    let auth_config = loaded_config().auth;
    assert!(
        !auth_config.jwt.secret.is_empty(),
        "JWT secret must be configured in test profile"
    );
    assert!(auth_config.jwt.secret.len() >= 32);
    assert!(auth_config.jwt.expiration_secs > 0);

    // Default cache config should have valid values
    let cache_config = loaded_config().system.infrastructure.cache;
    assert!(cache_config.default_ttl_secs > 0);
    assert!(cache_config.max_size > 0);
}

#[rstest]
#[case("localhost", 8080, false, "http://localhost:8080")]
#[case("api.example.com", 443, true, "https://api.example.com:443")]
fn server_url_generation(
    #[case] host: &str,
    #[case] port: u16,
    #[case] https: bool,
    #[case] expected_url: &str,
) {
    let mut config = default_server_config();
    config.network.host = host.to_owned();
    config.network.port = port;
    config.ssl.https = https;
    assert_eq!(config.get_base_url(), expected_url);
}

#[test]
fn test_cors_configuration() {
    // CORS disabled
    let mut no_cors = default_server_config();
    no_cors.cors.cors_enabled = false;
    no_cors.cors.cors_origins = vec![];
    assert!(!no_cors.cors.cors_enabled);
    assert!(no_cors.cors.cors_origins.is_empty());

    // CORS with specific origins
    let mut cors_config = default_server_config();
    cors_config.cors.cors_enabled = true;
    cors_config.cors.cors_origins = vec![
        "https://app.example.com".to_owned(),
        "https://admin.example.com".to_owned(),
    ];
    assert!(cors_config.cors.cors_enabled);
    assert_eq!(cors_config.cors.cors_origins.len(), 2);

    // Development preset has permissive CORS
    let dev_config = development_config();
    let (enabled, origins) = (
        dev_config.cors.cors_enabled,
        dev_config.cors.cors_origins.clone(),
    );
    assert!(enabled);
    assert!(origins.contains(&"*".to_owned()));
}

#[rstest]
#[case(120, 30)]
#[case(60, 10)]
fn timeout_configuration(#[case] request_secs: u64, #[case] connection_secs: u64) {
    let mut config = default_server_config();
    config.timeouts.request_timeout_secs = request_secs;
    config.timeouts.connection_timeout_secs = connection_secs;

    let request_timeout = config.request_timeout();
    let connection_timeout = config.connection_timeout();

    assert_eq!(request_timeout.as_secs(), request_secs);
    assert_eq!(connection_timeout.as_secs(), connection_secs);

    // Request timeout should generally be longer than connection timeout
    assert!(request_timeout > connection_timeout);
}
