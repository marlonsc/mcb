//! Configuration Validation Tests
//!
//! Tests for configuration validation across all config types.

use serial_test::serial;

use mcb_infrastructure::config::{CacheProvider, CacheSystemConfig, TestConfigBuilder};

fn loaded_config() -> Result<mcb_infrastructure::config::AppConfig, Box<dyn std::error::Error>> {
    TestConfigBuilder::new()
        .and_then(|b| b.build().map(|(config, _)| config))
        .map_err(|e| e.into())
}

#[test]
#[serial]
fn test_auth_config_jwt_secret_length() -> Result<(), Box<dyn std::error::Error>> {
    let default_auth = loaded_config()?.auth;
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
    Ok(())
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
    let default_cache = loaded_config()?.system.infrastructure.cache;
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
#[serial]
fn test_default_config_is_valid() -> Result<(), Box<dyn std::error::Error>> {
    let auth_config = loaded_config()?.auth;
    assert!(
        !auth_config.jwt.secret.is_empty(),
        "JWT secret must be configured in test profile"
    );
    assert!(auth_config.jwt.secret.len() >= 32);
    assert!(auth_config.jwt.expiration_secs > 0);

    // Default cache config should have valid values
    let cache_config = loaded_config()?.system.infrastructure.cache;
    assert!(cache_config.default_ttl_secs > 0);
    assert!(cache_config.max_size > 0);
    Ok(())
}
