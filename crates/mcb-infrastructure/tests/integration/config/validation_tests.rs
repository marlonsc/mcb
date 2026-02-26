//! Configuration Validation Tests
//!
//! Tests for configuration validation across all config types.

use rstest::rstest;
use serial_test::serial;
use std::path::PathBuf;

use mcb_infrastructure::config::{
    CacheProvider, CacheSystemConfig, ServerConfig, ServerConfigBuilder, ServerConfigPresets,
    ServerSslConfig, TestConfigBuilder,
};

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
    let config = ServerConfigBuilder::new().port(port).build();
    assert_eq!(config.network.port, port);
}

#[test]
fn server_config_address_parsing() {
    let config = ServerConfigBuilder::new()
        .host("127.0.0.1")
        .port(8080)
        .build();
    let addr = config.parse_address().unwrap();
    assert_eq!(addr.port(), 8080);
}

#[test]
#[serial]
fn test_auth_config_jwt_secret_length() {
    // Default config has empty secret - MUST be configured when auth is enabled
    // per ADR-025: fail-fast on missing configuration
    let default_auth = loaded_config().auth;
    assert!(
        default_auth.jwt.secret.is_empty(),
        "Default JWT secret should be empty (must be configured via settings.auth.jwt.secret in config YAML)"
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
        transport_mode: ServerConfigBuilder::new().build().transport_mode,
        network: ServerConfigBuilder::new().build().network,
        ssl: ServerSslConfig {
            https: true,
            ssl_cert_path: None,
            ssl_key_path: None,
        },
        timeouts: ServerConfigBuilder::new().build().timeouts,
        cors: ServerConfigBuilder::new().build().cors,
    };
    let result = https_no_ssl.validate_ssl();
    let err = result.expect_err("HTTPS without SSL paths should fail validation");
    assert!(
        err.to_string().contains("certificate path is required"),
        "error should mention certificate path: {err}"
    );

    // HTTPS with only cert path should fail
    let https_cert_only = ServerConfig {
        transport_mode: ServerConfigBuilder::new().build().transport_mode,
        network: ServerConfigBuilder::new().build().network,
        ssl: ServerSslConfig {
            https: true,
            ssl_cert_path: Some(PathBuf::from("/path/to/cert.pem")),
            ssl_key_path: None,
        },
        timeouts: ServerConfigBuilder::new().build().timeouts,
        cors: ServerConfigBuilder::new().build().cors,
    };
    let result = https_cert_only.validate_ssl();
    let err = result.expect_err("HTTPS with cert only should fail validation");
    assert!(
        err.to_string().contains("key path is required"),
        "error should mention key path: {err}"
    );

    // HTTP config doesn't require SSL
    let http_config = ServerConfig {
        transport_mode: ServerConfigBuilder::new().build().transport_mode,
        network: ServerConfigBuilder::new().build().network,
        ssl: ServerSslConfig {
            https: false,
            ssl_cert_path: None,
            ssl_key_path: None,
        },
        timeouts: ServerConfigBuilder::new().build().timeouts,
        cors: ServerConfigBuilder::new().build().cors,
    };
    let result = http_config.validate_ssl();
    result.expect("HTTP config should pass SSL validation");
}

#[test]
#[serial]
fn test_default_config_is_valid() {
    // Default server config should be parseable
    let server_config = ServerConfigBuilder::new().build();
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
    let dev_config = ServerConfigPresets::development();
    assert!(dev_config.parse_address().is_ok());
    assert!(dev_config.validate_ssl().is_ok());

    let test_config = ServerConfigPresets::testing();
    assert!(test_config.parse_address().is_ok());
    assert!(test_config.validate_ssl().is_ok());

    // Production preset has HTTPS but no SSL paths - that's expected for "template"
    // Users must provide real SSL paths for production
    let prod_config = ServerConfigPresets::production();
    assert!(prod_config.parse_address().is_ok());
    // Production config is a template, SSL paths must be added by user
    // So we just verify the address is valid

    // Default auth config - JWT secret is empty by design (must be configured)
    let auth_config = loaded_config().auth;
    assert!(
        auth_config.jwt.secret.is_empty(),
        "Default JWT secret should be empty per ADR-025"
    );
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
    let config = ServerConfigBuilder::new()
        .host(host)
        .port(port)
        .https(https)
        .build();
    assert_eq!(config.get_base_url(), expected_url);
}

#[test]
fn test_cors_configuration() {
    // CORS disabled
    let no_cors = ServerConfigBuilder::new().cors(false, vec![]).build();
    assert!(!no_cors.cors.cors_enabled);
    assert!(no_cors.cors.cors_origins.is_empty());

    // CORS with specific origins
    let cors_config = ServerConfigBuilder::new()
        .cors(
            true,
            vec![
                "https://app.example.com".to_owned(),
                "https://admin.example.com".to_owned(),
            ],
        )
        .build();
    assert!(cors_config.cors.cors_enabled);
    assert_eq!(cors_config.cors.cors_origins.len(), 2);

    // Development preset has permissive CORS
    let dev_config = ServerConfigPresets::development();
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
    let config = ServerConfigBuilder::new()
        .request_timeout(request_secs)
        .connection_timeout(connection_secs)
        .build();

    let request_timeout = config.request_timeout();
    let connection_timeout = config.connection_timeout();

    assert_eq!(request_timeout.as_secs(), request_secs);
    assert_eq!(connection_timeout.as_secs(), connection_secs);

    // Request timeout should generally be longer than connection timeout
    assert!(request_timeout > connection_timeout);
}
