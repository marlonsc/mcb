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
use mcb_infrastructure::constants::http::DEFAULT_HTTP_PORT;
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
        Ok("test-subscription".to_string())
    }
}

fn test_event_bus() -> Arc<dyn EventBusProvider> {
    Arc::new(TestEventBus)
}

/// Create test config with auth disabled (avoids JWT secret validation)
fn test_config() -> AppConfig {
    let mut config = AppConfig::default();
    config.auth.enabled = false;
    config
}

#[tokio::test]
async fn test_config_watcher_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    // Create initial config file with auth disabled
    let initial_config = test_config();
    let loader = ConfigLoader::new();
    loader.save_to_file(&initial_config, &config_path).unwrap();

    let watcher = ConfigWatcher::new(config_path, initial_config, test_event_bus())
        .await
        .unwrap();
    let config = watcher.get_config().await;

    assert_eq!(config.server.network.port, DEFAULT_HTTP_PORT);
}

#[tokio::test]
async fn test_manual_reload() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let initial_config = test_config();
    let loader = ConfigLoader::new();
    loader.save_to_file(&initial_config, &config_path).unwrap();

    let watcher = ConfigWatcher::new(config_path.clone(), initial_config, test_event_bus())
        .await
        .unwrap();

    // Modify config file with new port (keep auth disabled)
    let mut new_config = test_config();
    new_config.server.network.port = 9999;
    loader.save_to_file(&new_config, &config_path).unwrap();

    // Small delay to ensure file is written
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Reload manually
    let reloaded_config = watcher.reload().await.unwrap();
    assert_eq!(reloaded_config.server.network.port, 9999);

    // Check current config was updated
    let current_config = watcher.get_config().await;
    assert_eq!(current_config.server.network.port, 9999);

    // Drop watcher explicitly before temp_dir to avoid race conditions
    drop(watcher);
}

#[test]
fn test_watcher_builder() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let builder = ConfigWatcherBuilder::new()
        .with_config_path(&config_path)
        .with_initial_config(test_config())
        .with_event_bus(test_event_bus());

    // Builder should validate that config file exists
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(builder.build());

    assert!(result.is_err()); // File doesn't exist yet
}

// Note: should_reload_config is private, so we test the public API instead
#[test]
fn test_config_watcher_exists() {
    // Test that ConfigWatcher type exists and can be referenced
    let _ = std::any::type_name::<ConfigWatcher>();
}
