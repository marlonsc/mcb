//! Tests verifying Figment configuration pattern compliance (ADR-025)
#![allow(unsafe_code)]
#![allow(clippy::mem_forget)]
#![allow(clippy::used_underscore_binding)]

use crate::utils::env_vars::EnvVarGuard;
use mcb_infrastructure::config::loader::ConfigLoader;
use rstest::rstest;
use rstest::*;
use serial_test::serial;

#[fixture]
fn auth_disabled() -> EnvVarGuard {
    EnvVarGuard::new(&[("MCP__AUTH__ENABLED", "false")])
}

#[fixture]
fn clean_env() -> EnvVarGuard {
    EnvVarGuard::new(&[])
}

#[rstest]
#[serial]
fn test_mcp_prefix_env_vars_loaded(_auth_disabled: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("MCP__PROVIDERS__EMBEDDING__PROVIDER", "test-provider")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert_eq!(
        config.providers.embedding.provider,
        Some("test-provider".to_owned()),
        "MCP__ prefixed env vars should be loaded by Figment"
    );
}

#[rstest]
#[serial]
fn test_old_mcb_prefix_not_loaded(_auth_disabled: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("MCB_ADMIN_API_KEY", "old-key-value")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert_eq!(
        config.auth.admin.key, None,
        "Old MCB_ prefix should NOT be recognized (ADR-025 breaking change)"
    );
}

#[rstest]
#[serial]
fn test_new_admin_key_loaded(_auth_disabled: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("MCP__AUTH__ADMIN__KEY", "new-key-value")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert_eq!(
        config.auth.admin.key,
        Some("new-key-value".to_owned()),
        "MCP__AUTH__ADMIN__KEY should be loaded by Figment"
    );
}

#[rstest]
#[serial]
fn test_jwt_secret_required_when_auth_enabled() {
    let _guard = EnvVarGuard::new(&[("MCP__AUTH__ENABLED", "true")]);
    // Deliberately NOT setting MCP__AUTH__JWT__SECRET

    let result = ConfigLoader::new().load();

    assert!(
        result.is_err(),
        "Config should fail validation when auth.enabled=true but JWT secret is empty"
    );

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("JWT") || err.contains("secret"),
        "Error message should mention JWT secret requirement, got: {err}"
    );
}

#[rstest]
#[serial]
fn test_watching_enabled_via_figment(_auth_disabled: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED", "false")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert!(
        !config.system.data.sync.watching_enabled,
        "watching_enabled should be loaded from MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED"
    );
}

#[rstest]
#[serial]
fn test_legacy_disable_watching_not_supported(_auth_disabled: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("DISABLE_CONFIG_WATCHING", "true")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert!(
        config.system.data.sync.watching_enabled,
        "DISABLE_CONFIG_WATCHING should NOT affect watching_enabled (use MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED)"
    );
}

#[rstest]
#[serial]
fn test_auth_disabled_by_default_loads_without_jwt_secret() {
    EnvVarGuard::remove(&["MCP__AUTH__ENABLED", "MCP__AUTH__JWT__SECRET"]);

    let result = ConfigLoader::new().load();

    assert!(
        result.is_ok(),
        "Config should load when auth.enabled=false (default), got error: {:?}",
        result.err()
    );
}
