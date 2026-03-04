//! Configuration Validation Tests
//!
//! Tests for configuration validation across all config types.
//! All validation goes through `ConfigProvider` (CA/DI pattern).

use serial_test::serial;

use mcb_domain::ports::ConfigProvider;
use mcb_domain::registry::config::{ConfigProviderConfig, resolve_config_provider};
use mcb_infrastructure::config::app::AppConfig;
use mcb_infrastructure::config::infrastructure::{CacheProvider, CacheSystemConfig};
use rstest::rstest;

use super::test_builder::TestConfigBuilder;

/// Resolve the default `ConfigProvider` via CA/DI registry.
#[allow(clippy::expect_used)]
fn config_provider() -> std::sync::Arc<dyn ConfigProvider> {
    resolve_config_provider(&ConfigProviderConfig::new(
        mcb_domain::utils::config::DEFAULT_PROVIDER,
    ))
    .expect("loco_yaml config provider must be registered")
}

fn loaded_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    TestConfigBuilder::new()
        .and_then(|b| b.build().map(|(config, _)| config))
        .map_err(Into::into)
}

#[rstest]
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

#[rstest]
#[test]
#[serial]
fn test_cache_config_ttl_when_enabled() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

#[rstest]
#[test]
#[serial]
fn test_default_config_is_valid() -> Result<(), Box<dyn std::error::Error>> {
    // Load via CA/DI and validate
    let config_any = config_provider().load_config()?;
    let config = config_any
        .downcast::<AppConfig>()
        .map_err(|_| "unexpected type")?;

    assert!(
        !config.auth.jwt.secret.is_empty(),
        "JWT secret must be configured in test profile"
    );
    assert!(config.auth.jwt.secret.len() >= 32);
    assert!(config.auth.jwt.expiration_secs > 0);

    // Default cache config should have valid values
    assert!(config.system.infrastructure.cache.default_ttl_secs > 0);
    assert!(config.system.infrastructure.cache.max_size > 0);
    Ok(())
}
