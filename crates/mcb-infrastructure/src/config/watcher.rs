//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#configuration)
//!
//! Configuration file watcher for hot-reloading
//!
//! Provides automatic configuration reloading when the configuration file changes.

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::config::loader::ConfigLoader;
use crate::error_ext::ErrorContext;
use crate::logging::log_config_loaded;
use mcb_domain::error::{Error, Result};
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::EventBusProvider;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::runtime::Handle;
use tokio::sync::RwLock;

/// Configuration watcher for hot-reloading
pub struct ConfigWatcher {
    config_path: PathBuf,
    loader: ConfigLoader,
    current_config: Arc<RwLock<AppConfig>>,
    event_bus: Arc<dyn EventBusProvider>,
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be created.
    pub async fn new(
        config_path: PathBuf,
        initial_config: AppConfig,
        event_bus: Arc<dyn EventBusProvider>,
    ) -> Result<Self> {
        let current_config = Arc::new(RwLock::new(initial_config));
        let loader = ConfigLoader::new().with_config_path(&config_path);

        // Create file watcher
        let mut watcher = Self::create_file_watcher(
            config_path.clone(),
            Arc::clone(&current_config),
            loader.clone(),
            Arc::clone(&event_bus),
        )
        .await?;

        // Watch the configuration file
        watcher
            .watch(&config_path, RecursiveMode::NonRecursive)
            .context("Failed to watch configuration file")?;

        Ok(Self {
            config_path,
            loader,
            current_config,
            event_bus,
            _watcher: watcher,
        })
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> AppConfig {
        self.current_config.read().await.clone()
    }

    /// Manually trigger a configuration reload
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be loaded.
    pub async fn reload(&self) -> Result<AppConfig> {
        let new_config = self.loader.load()?;

        // Update current config
        *self.current_config.write().await = new_config.clone();

        Self::publish_config_reloaded(self.event_bus.as_ref()).await;

        log_config_loaded(&self.config_path, true);

        Ok(new_config)
    }

    /// Get the configuration file path
    #[must_use]
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Create the file watcher
    async fn create_file_watcher(
        config_path: PathBuf,
        current_config: Arc<RwLock<AppConfig>>,
        loader: ConfigLoader,
        event_bus: Arc<dyn EventBusProvider>,
    ) -> Result<RecommendedWatcher> {
        let config_path_clone = config_path.clone();
        // Capture the Tokio runtime handle to use from the notify callback thread
        let runtime_handle = Handle::current();

        let watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let config_path = config_path_clone.clone();
                let current_config = Arc::clone(&current_config);
                let loader = loader.clone();
                let event_bus = Arc::clone(&event_bus);

                // Use the captured runtime handle to spawn tasks from the notify thread
                runtime_handle.spawn(async move {
                    match res {
                        Ok(event) => {
                            if Self::should_reload_config(&event) {
                                Self::handle_config_change(
                                    config_path,
                                    current_config,
                                    loader,
                                    event_bus,
                                )
                                .await;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "File watch error");
                        }
                    }
                });
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        Ok(watcher)
    }

    /// Check if the file event should trigger a config reload
    fn should_reload_config(event: &Event) -> bool {
        // Only reload on write or create events
        matches!(
            event.kind,
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
                | notify::EventKind::Modify(notify::event::ModifyKind::Any)
                | notify::EventKind::Create(_)
        )
    }

    /// Handle configuration file change
    async fn handle_config_change(
        config_path: PathBuf,
        current_config: Arc<RwLock<AppConfig>>,
        loader: ConfigLoader,
        event_bus: Arc<dyn EventBusProvider>,
    ) {
        // Add a small delay to avoid reading partially written files
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        match loader.load() {
            Ok(new_config) => {
                // Update current config
                *current_config.write().await = new_config;
                Self::publish_config_reloaded(event_bus.as_ref()).await;

                log_config_loaded(&config_path, true);
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to reload configuration");

                log_config_loaded(&config_path, false);
            }
        }
    }

    async fn publish_config_reloaded(event_bus: &dyn EventBusProvider) {
        if let Err(e) = event_bus
            .publish_event(DomainEvent::ConfigReloaded {
                section: "all".to_owned(),
                timestamp: chrono::Utc::now().timestamp(),
            })
            .await
        {
            tracing::warn!(error = %e, "Failed to publish config reload event");
        }
    }
}

/// Configuration watcher builder
pub struct ConfigWatcherBuilder {
    config_path: Option<PathBuf>,
    initial_config: Option<AppConfig>,
    event_bus: Option<Arc<dyn EventBusProvider>>,
}

impl ConfigWatcherBuilder {
    /// Create a new configuration watcher builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_path: None,
            initial_config: None,
            event_bus: None,
        }
    }

    /// Set the configuration file path
    #[must_use]
    pub fn with_config_path<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the initial configuration
    #[must_use]
    pub fn with_initial_config(mut self, config: AppConfig) -> Self {
        self.initial_config = Some(config);
        self
    }

    /// Set the event bus provider used for config notifications
    #[must_use]
    pub fn with_event_bus(mut self, event_bus: Arc<dyn EventBusProvider>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Build the configuration watcher
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or watcher initialization fails.
    pub async fn build(self) -> Result<ConfigWatcher> {
        let config_path = self.config_path.ok_or_else(|| Error::Configuration {
            message: "Configuration file path is required".to_owned(),
            source: None,
        })?;

        let initial_config = self.initial_config.ok_or_else(|| Error::Configuration {
            message: "Initial configuration is required".to_owned(),
            source: None,
        })?;

        let event_bus = self.event_bus.ok_or_else(|| Error::Configuration {
            message: "Event bus is required".to_owned(),
            source: None,
        })?;

        ConfigWatcher::new(config_path, initial_config, event_bus).await
    }
}

/// Returns default `ConfigWatcherBuilder` for creating config file watchers
impl Default for ConfigWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration watcher utilities
pub struct ConfigWatcherUtils;

impl ConfigWatcherUtils {
    /// Create a watcher that automatically reloads on file changes
    ///
    /// Before calling this, check if watching is enabled via
    /// `config.system.data.sync.watching_enabled`. This method
    /// assumes the caller has already verified watching should proceed.
    ///
    /// # Errors
    ///
    /// Returns an error if the file watcher cannot be initialized.
    pub async fn watch_config_file(
        config_path: PathBuf,
        initial_config: AppConfig,
        event_bus: Arc<dyn EventBusProvider>,
    ) -> Result<ConfigWatcher> {
        ConfigWatcher::new(config_path, initial_config, event_bus).await
    }
}
