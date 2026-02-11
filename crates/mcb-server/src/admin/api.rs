//! Admin API server
//!
//! HTTP server builder for admin API routes.
//!
//! ## Wiring
//!
//! The default startup path in `init.rs` mounts admin routes into the unified HTTP server.
//! `AdminApi` remains available for explicit embedding scenarios.
//!
//! - **Config (GET/PATCH /config)**: Call `.with_config_watcher(config_watcher, config_path)` so
//!   `AdminState` has a config watcher; otherwise GET /config returns 503 and PATCH is unusable.
//! - **Browse (GET /collections, ...)**: Call `.with_browse_state(browse_state)` so browse routes
//!   are mounted; otherwise /collections returns 404.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::config::watcher::ConfigWatcher;
use rocket::config::{Config as RocketConfig, LogLevel};

use super::auth::AdminAuthConfig;
use super::browse_handlers::BrowseState;
use super::handlers::AdminState;
use super::routes::admin_rocket;

/// Admin API server configuration
#[derive(Debug, Clone)]
pub struct AdminApiConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
}

impl Default for AdminApiConfig {
    fn default() -> Self {
        let config = ConfigLoader::new()
            .load()
            .expect("AdminApiConfig::default requires loadable configuration file");
        Self {
            host: config.server.network.host,
            port: config.server.network.port,
        }
    }
}

impl AdminApiConfig {
    /// Create config for localhost with specified port
    pub fn localhost(port: u16) -> Self {
        let config = ConfigLoader::new()
            .load()
            .expect("AdminApiConfig::localhost requires loadable configuration file");
        Self {
            host: config.server.network.host,
            port,
        }
    }

    /// Get the Rocket configuration
    pub fn rocket_config(&self) -> RocketConfig {
        let address: IpAddr = self
            .host
            .parse()
            .expect("Invalid admin host in configuration");
        RocketConfig {
            address,
            port: self.port,
            log_level: LogLevel::Normal,
            ..RocketConfig::default()
        }
    }
}

/// Admin API server
pub struct AdminApi {
    config: AdminApiConfig,
    state: AdminState,
    auth_config: Arc<AdminAuthConfig>,
    browse_state: Option<BrowseState>,
}

impl AdminApi {
    /// Create a new admin API server
    pub fn new(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
    ) -> Self {
        Self {
            config,
            state: AdminState {
                metrics,
                indexing,
                config_watcher: None,
                current_config: mcb_infrastructure::config::AppConfig::default(),
                config_path: None,
                shutdown_coordinator: None,
                shutdown_timeout_secs: 30,
                event_bus,
                service_manager: None,
                cache: None,
                project_workflow: None,
            },
            auth_config: Arc::new(AdminAuthConfig::default()),
            browse_state: None,
        }
    }

    /// Create a new admin API server with authentication
    pub fn with_auth(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
        auth_config: AdminAuthConfig,
    ) -> Self {
        Self {
            config,
            state: AdminState {
                metrics,
                indexing,
                config_watcher: None,
                current_config: mcb_infrastructure::config::AppConfig::default(),
                config_path: None,
                shutdown_coordinator: None,
                shutdown_timeout_secs: 30,
                event_bus,
                service_manager: None,
                cache: None,
                project_workflow: None,
            },
            auth_config: Arc::new(auth_config),
            browse_state: None,
        }
    }

    /// Create a new admin API server with configuration watcher support
    pub fn with_config_watcher(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        config_watcher: Arc<ConfigWatcher>,
        config_path: PathBuf,
        event_bus: Arc<dyn EventBusProvider>,
        auth_config: AdminAuthConfig,
    ) -> Self {
        Self {
            config,
            state: AdminState {
                metrics,
                indexing,
                config_watcher: Some(config_watcher),
                current_config: mcb_infrastructure::config::AppConfig::default(),
                config_path: Some(config_path),
                shutdown_coordinator: None,
                shutdown_timeout_secs: 30,
                event_bus,
                service_manager: None,
                cache: None,
                project_workflow: None,
            },
            auth_config: Arc::new(auth_config),
            browse_state: None,
        }
    }

    /// Set the browse state for code browsing functionality
    ///
    /// When set, enables the browse API endpoints for navigating
    /// indexed collections, files, and code chunks.
    pub fn with_browse_state(mut self, browse_state: BrowseState) -> Self {
        self.browse_state = Some(browse_state);
        self
    }

    /// Start the admin API server
    ///
    /// Returns a handle that can be used to gracefully shutdown the server.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rocket_config = self.config.rocket_config();

        tracing::info!(
            "Admin API server listening on {}:{}",
            rocket_config.address,
            rocket_config.port
        );

        let rocket =
            admin_rocket(self.state, self.auth_config, self.browse_state).configure(rocket_config);

        rocket.launch().await.map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Rocket launch failed: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        Ok(())
    }

    /// Start the admin API server with graceful shutdown
    ///
    /// Note: Rocket handles graceful shutdown internally via Ctrl+C or SIGTERM.
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rocket_config = self.config.rocket_config();

        tracing::info!(
            "Admin API server listening on {}:{}",
            rocket_config.address,
            rocket_config.port
        );

        let rocket = admin_rocket(self.state, self.auth_config, self.browse_state)
            .configure(rocket_config)
            .ignite()
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!(
                    "Rocket ignite failed: {}",
                    e
                ))) as Box<dyn std::error::Error + Send + Sync>
            })?;

        // Spawn a task to handle the external shutdown signal
        let shutdown_handle = rocket.shutdown();
        tokio::spawn(async move {
            shutdown_signal.await;
            shutdown_handle.notify();
        });

        rocket.launch().await.map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Rocket launch failed: {}",
                e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

        Ok(())
    }
}
