//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Configuration propagation for runtime config changes
//!
//! Handles propagating configuration changes to services that support hot-reload.
//! Uses the `ConfigWatcher` event subscription mechanism.

use std::sync::Arc;

use futures::StreamExt;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::{DomainEventStream, EventBusProvider};
use mcb_domain::{debug, error, info, warn};
use mcb_infrastructure::config::watcher::ConfigWatcher;

/// Configuration change handler callback type
pub type ConfigChangeCallback = Box<dyn Fn(&mcb_infrastructure::config::AppConfig) + Send + Sync>;

/// Configuration propagator that handles runtime config changes
pub struct ConfigPropagator {
    /// Registered config change callbacks (visible for testing)
    pub callbacks: Vec<ConfigChangeCallback>,
}

impl ConfigPropagator {
    /// Create a new config propagator
    #[must_use]
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    /// Register a callback to be called when config changes
    #[must_use]
    pub fn on_config_change(mut self, callback: ConfigChangeCallback) -> Self {
        self.callbacks.push(callback);
        self
    }

    /// Start listening for config changes from the watcher
    ///
    /// This spawns a background task that processes config change events.
    /// Returns immediately after spawning the task.
    pub fn start(
        self,
        config_watcher: Arc<ConfigWatcher>,
        event_bus: Arc<dyn EventBusProvider>,
    ) -> PropagatorHandle {
        let callbacks = Arc::new(self.callbacks);

        let handle = tokio::spawn(async move {
            let event_stream = match event_bus.subscribe_events().await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("ConfigPropagator", "Failed to subscribe to event bus", &e);
                    return;
                }
            };

            Self::run_event_loop(event_stream, config_watcher, callbacks).await;
        });

        PropagatorHandle { handle }
    }

    /// Run the config change event loop
    async fn run_event_loop(
        mut event_stream: DomainEventStream,
        config_watcher: Arc<ConfigWatcher>,
        callbacks: Arc<Vec<ConfigChangeCallback>>,
    ) {
        info!(
            "ConfigPropagator",
            "Config propagator started, listening for config changes"
        );

        while let Some(event) = event_stream.next().await {
            if let DomainEvent::ConfigReloaded { .. } = event {
                Self::handle_reload_event(&config_watcher, &callbacks).await;
            } else {
                debug!(
                    "ConfigPropagator",
                    "Ignoring non-config domain event in propagator",
                    &format!("{event:?}")
                );
            }
        }

        warn!(
            "ConfigPropagator",
            "Event bus stream closed, stopping config propagator"
        );
        info!("ConfigPropagator", "Config propagator stopped");
    }

    /// Handle a config reload domain event
    async fn handle_reload_event(
        config_watcher: &ConfigWatcher,
        callbacks: &[ConfigChangeCallback],
    ) {
        let config = config_watcher.get_config().await;

        info!(
            "ConfigPropagator",
            "Configuration reloaded, propagating to listeners",
            &callbacks.len()
        );
        debug!(
            "ConfigPropagator",
            "New configuration applied",
            &format!(
                "transport_mode={:?} cache_enabled={}",
                config.server.transport_mode, config.system.infrastructure.cache.enabled
            )
        );

        // Call all registered callbacks
        for callback in callbacks {
            callback(&config);
        }

        info!("ConfigPropagator", "Configuration propagation complete");
    }
}

impl Default for ConfigPropagator {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to the running config propagator task
pub struct PropagatorHandle {
    /// The tokio task handle (visible for testing)
    pub handle: tokio::task::JoinHandle<()>,
}

impl PropagatorHandle {
    /// Wait for the propagator task to complete
    ///
    /// # Errors
    /// Returns an error if the underlying task panics or is cancelled.
    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        self.handle.await
    }

    /// Abort the propagator task
    pub fn abort(self) {
        self.handle.abort();
    }

    /// Check if the propagator task is still running
    #[must_use]
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }
}

/// Pre-built config change callbacks for common services
pub mod callbacks {
    use mcb_domain::info;

    use super::ConfigChangeCallback;

    /// Create a callback that logs all config changes
    #[must_use]
    pub fn logging_callback() -> ConfigChangeCallback {
        Box::new(|config| {
            let detail = format!(
                "transport_mode={:?} http_port={} cache_enabled={}",
                config.server.transport_mode,
                config.server.network.port,
                config.system.infrastructure.cache.enabled,
            );
            info!("ConfigPropagator", "Configuration change logged", &detail);
        })
    }

    /// Create a callback that updates logging level (if supported)
    #[must_use]
    pub fn log_level_callback() -> ConfigChangeCallback {
        Box::new(|config| {
            // Note: Changing log level at runtime requires tracing_subscriber reload
            // which is not straightforward. This logs the new level for awareness.
            info!(
                "ConfigPropagator",
                "Log level configuration changed (requires restart to take effect)",
                &config.logging.level
            );
        })
    }
}
