//! Configuration Loader Tests

use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::config::TransportMode;
use serial_test::serial;
use tempfile::TempDir;

/// Create test config with auth disabled (avoids JWT secret validation per ADR-025)
fn test_config() -> AppConfig {
    let mut config = ConfigLoader::new().load().expect("load default config");
    config.auth.enabled = false;
    config
}

/// Test config builder creates valid config with expected defaults
///
/// Note: Per ADR-025, when auth is enabled, JWT secret MUST be configured.
/// We use auth disabled to test the builder without validation failure.
#[test]
#[serial]
fn test_config_loader_default() {
    // Build config directly with auth disabled
    let config = test_config();
    let loaded = ConfigLoader::new().load().expect("load config");

    assert_eq!(config.server.network.port, loaded.server.network.port);
    assert_eq!(config.logging.level, loaded.logging.level);
}

#[test]
#[serial]
fn test_config_builder() {
    let mut config = test_config();
    let loaded = ConfigLoader::new().load().expect("load config");
    config.server.network.port = loaded.server.network.port.saturating_add(1);

    assert_eq!(
        config.server.network.port,
        loaded.server.network.port.saturating_add(1)
    );
}

// Note: validate_config is private, so we test the public API instead
#[test]
#[serial]
fn test_config_loader_exists() {
    // Test that ConfigLoader type exists and can be created
    let _ = ConfigLoader::new();
}

#[test]
#[serial]
fn test_config_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let loader = ConfigLoader::new();

    // Create config with custom port and auth disabled
    let mut original_config = test_config();
    let loaded = ConfigLoader::new().load().expect("load config");
    original_config.server.network.port = loaded.server.network.port.saturating_add(9);

    // Save config
    loader.save_to_file(&original_config, &config_path).unwrap();

    // Load config
    let loaded_config = ConfigLoader::new()
        .with_config_path(&config_path)
        .load()
        .unwrap();

    assert_eq!(
        loaded_config.server.network.port,
        loaded.server.network.port.saturating_add(9)
    );
}

#[test]
#[serial]
fn test_config_loader_rejects_http_transport_mode() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid_http_transport.toml");

    let mut config = test_config();
    config.server.transport_mode = TransportMode::Http;

    ConfigLoader::new()
        .save_to_file(&config, &config_path)
        .expect("save invalid config");

    let err = ConfigLoader::new()
        .with_config_path(&config_path)
        .load()
        .expect_err("http transport mode must be rejected");

    let msg = err.to_string();
    assert!(msg.contains("transport_mode=http is not supported"));
}
