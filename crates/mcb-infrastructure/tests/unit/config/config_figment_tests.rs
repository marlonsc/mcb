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
fn clean_env() -> EnvVarGuard {
    EnvVarGuard::new(&[])
}

#[rstest]
#[serial]
fn test_old_mcb_prefix_not_loaded(_clean_env: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("MCB_ADMIN_API_KEY", "old-key-value")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert_eq!(
        config.auth.admin.key, None,
        "Old MCB_ prefix should NOT be recognized (ADR-025 breaking change)"
    );
}

#[rstest]
#[serial]
fn test_legacy_disable_watching_not_supported(_clean_env: EnvVarGuard) {
    let _guard = EnvVarGuard::new(&[("DISABLE_CONFIG_WATCHING", "true")]);

    let config = ConfigLoader::new().load().expect("Should load config");

    assert!(
        config.system.data.sync.watching_enabled,
        "DISABLE_CONFIG_WATCHING should NOT affect watching_enabled (legacy env var is dead)"
    );
}

#[rstest]
#[serial]
fn test_auth_disabled_by_default_loads_without_jwt_secret() {
    let config = ConfigLoader::new().load().expect("Should load config");

    assert!(
        !config.auth.enabled,
        "Auth should be disabled by default in development.yaml"
    );
}

#[rstest]
#[serial]
fn test_jwt_secret_required_when_auth_enabled() {
    // Load valid config, then mutate to auth.enabled=true with empty secret
    let mut config = ConfigLoader::new().load().expect("Should load base config");
    config.auth.enabled = true;
    config.auth.jwt.secret = String::new();

    let result = ConfigLoader::validate_for_test(&config);

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
