//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
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

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::ports::EventBusProvider;
use mcb_domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_infrastructure::config::watcher::ConfigWatcher;

use super::auth::AdminAuthConfig;
use super::browse_handlers::BrowseState;
use super::handlers::AdminState;
use crate::constants::limits::DEFAULT_SHUTDOWN_TIMEOUT_SECS;
use crate::utils::config::load_startup_config_or_default;

fn load_current_config() -> mcb_infrastructure::config::AppConfig {
    load_startup_config_or_default()
}

fn build_admin_state(
    metrics: Arc<dyn PerformanceMetricsInterface>,
    indexing: Arc<dyn IndexingOperationsInterface>,
    event_bus: Arc<dyn EventBusProvider>,
    config_watcher: Option<Arc<ConfigWatcher>>,
    config_path: Option<PathBuf>,
) -> AdminState {
    AdminState {
        metrics,
        indexing,
        config_watcher,
        current_config: load_current_config(),
        config_path,
        shutdown_coordinator: None,
        shutdown_timeout_secs: DEFAULT_SHUTDOWN_TIMEOUT_SECS,
        event_bus,
        service_manager: None,
        cache: None,
        project_workflow: None,
        vcs_entity: None,
        plan_entity: None,
        issue_entity: None,
        org_entity: None,
        tool_handlers: None,
    }
}

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
        let config = load_startup_config_or_default();
        Self {
            host: config.server.network.host,
            port: config.server.network.port,
        }
    }
}

impl AdminApiConfig {
    /// Create config for localhost with specified port
    #[must_use]
    pub fn localhost(port: u16) -> Self {
        let config = load_startup_config_or_default();
        Self {
            host: config.server.network.host,
            port,
        }
    }
}

/// Admin API server
pub struct AdminApi {
    pub(super) config: AdminApiConfig,
    pub(super) state: AdminState,
    pub(super) auth_config: Arc<AdminAuthConfig>,
    pub(super) browse_state: Option<BrowseState>,
}

/// Core service dependencies needed by `AdminApi` builders.
pub struct AdminApiServices {
    /// Metrics service used by admin endpoints.
    pub metrics: Arc<dyn PerformanceMetricsInterface>,
    /// Indexing operations used by admin indexing endpoints.
    pub indexing: Arc<dyn IndexingOperationsInterface>,
    /// Event bus used for lifecycle and integration events.
    pub event_bus: Arc<dyn EventBusProvider>,
}

/// Configuration watcher wiring for admin config endpoints.
pub struct AdminConfigWatcherConfig {
    /// Shared config watcher instance.
    pub watcher: Arc<ConfigWatcher>,
    /// Path to the loaded configuration file.
    pub config_path: PathBuf,
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
            state: build_admin_state(metrics, indexing, event_bus, None, None),
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
            state: build_admin_state(metrics, indexing, event_bus, None, None),
            auth_config: Arc::new(auth_config),
            browse_state: None,
        }
    }

    /// Create a new admin API server with configuration watcher support
    #[must_use]
    pub fn with_config_watcher(
        config: AdminApiConfig,
        services: AdminApiServices,
        watcher_config: AdminConfigWatcherConfig,
        auth_config: AdminAuthConfig,
    ) -> Self {
        Self {
            config,
            state: build_admin_state(
                services.metrics,
                services.indexing,
                services.event_bus,
                Some(watcher_config.watcher),
                Some(watcher_config.config_path),
            ),
            auth_config: Arc::new(auth_config),
            browse_state: None,
        }
    }

    /// Set the browse state for code browsing functionality
    ///
    /// When set, enables the browse API endpoints for navigating
    /// indexed collections, files, and code chunks.
    #[must_use]
    pub fn with_browse_state(mut self, browse_state: BrowseState) -> Self {
        self.browse_state = Some(browse_state);
        self
    }
}
