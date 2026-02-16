//! Configuration Watcher Tests

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::config::loader::ConfigLoader;
use mcb_infrastructure::config::watcher::{ConfigWatcher, ConfigWatcherBuilder};
use serial_test::serial;
use tempfile::TempDir;

#[derive(Default)]
struct TestEventBus;

#[async_trait]
impl EventBusProvider for TestEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        Ok(Box::pin(stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("test-subscription".to_owned())
    }
}

fn test_event_bus() -> Arc<dyn EventBusProvider> {
    Arc::new(TestEventBus)
}

/// Create test config with auth disabled (avoids JWT secret validation)
///
/// # Errors
///
/// Returns an error if the default config cannot be loaded.
fn test_config() -> Result<AppConfig> {
    let mut config = ConfigLoader::new().load()?;
    config.auth.enabled = false;
    Ok(config)
}

#[tokio::test]
#[serial]
async fn test_config_watcher_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");

    // Create initial config file with auth disabled
    let initial_config = test_config()?;
    let expected_port = initial_config.server.network.port;
    let loader = ConfigLoader::new();
    loader.save_to_file(&initial_config, &config_path)?;

    let watcher = ConfigWatcher::new(config_path, initial_config, test_event_bus()).await?;
    let config = watcher.get_config().await;

    assert_eq!(config.server.network.port, expected_port);
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_manual_reload() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");

    let initial_config = test_config()?;
    let loader = ConfigLoader::new();
    loader.save_to_file(&initial_config, &config_path)?;

    let watcher = ConfigWatcher::new(config_path.clone(), initial_config, test_event_bus()).await?;

    // Modify config file with new port (keep auth disabled)
    let mut new_config = test_config()?;
    new_config.server.network.port = 9999;
    loader.save_to_file(&new_config, &config_path)?;

    // Small delay to ensure file is written
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Reload manually
    let reloaded_config = watcher.reload().await?;
    assert_eq!(reloaded_config.server.network.port, 9999);

    // Check current config was updated
    let current_config = watcher.get_config().await;
    assert_eq!(current_config.server.network.port, 9999);

    // Drop watcher explicitly before temp_dir to avoid race conditions
    drop(watcher);
    Ok(())
}

#[test]
#[serial]
fn test_watcher_builder() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");

    let builder = ConfigWatcherBuilder::new()
        .with_config_path(&config_path)
        .with_initial_config(test_config()?)
        .with_event_bus(test_event_bus());

    // Builder should validate that config file exists
    let result = tokio::runtime::Runtime::new()?.block_on(builder.build());

    assert!(result.is_err()); // File doesn't exist yet
    Ok(())
}

// Note: should_reload_config is private, so we test the public API instead
#[test]
#[serial]
fn test_config_watcher_exists() {
    // Test that ConfigWatcher type exists and can be referenced
    let _ = std::any::type_name::<ConfigWatcher>();
}
