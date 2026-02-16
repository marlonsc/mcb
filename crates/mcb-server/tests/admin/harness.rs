//! Admin Test Harness
//!
//! Shared test infrastructure for admin API tests. Eliminates duplication
//! of `TestEventBus`, `AdminState` construction, and fixture setup across
//! admin test modules.
//!
//! # Usage
//!
//! ```rust
//! use super::harness::AdminTestHarness;
//!
//! let (client, metrics, indexing) = AdminTestHarness::new()
//!     .with_recorded_metrics(&[(100, true, true), (200, false, false)], 5)
//!     .build_client()
//!     .await;
//! ```

use std::sync::Arc;

use mcb_domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_providers::events::TokioEventBusProvider;
use mcb_server::admin::{auth::AdminAuthConfig, handlers::AdminState, routes::admin_rocket};
use rocket::local::asynchronous::Client;

// ============================================================================
// Admin Test Harness Builder
// ============================================================================

/// Builder for creating admin test state and Rocket client.
///
/// Provides a fluent API for configuring `AdminState` with metrics,
/// indexing operations, authentication, and other settings.
pub struct AdminTestHarness {
    metrics: Arc<AtomicPerformanceMetrics>,
    indexing: Arc<DefaultIndexingOperations>,
    auth_config: AdminAuthConfig,
    shutdown_timeout_secs: u64,
}

impl Default for AdminTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl AdminTestHarness {
    /// Create a new harness with default (empty) state.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(AtomicPerformanceMetrics::new()),
            indexing: Arc::new(DefaultIndexingOperations::new()),
            auth_config: AdminAuthConfig::default(),
            shutdown_timeout_secs: 30,
        }
    }

    /// Record query metrics: each tuple is (`latency_ms`, success, `cache_hit`).
    pub fn with_recorded_metrics(self, queries: &[(u64, bool, bool)], connections: i64) -> Self {
        for &(latency_ms, success, cache_hit) in queries {
            self.metrics.record_query(latency_ms, success, cache_hit);
        }
        self.metrics.update_active_connections(connections);
        self
    }

    /// Start indexing operations: each tuple is (`collection_name`, `total_files`).
    pub fn with_indexing_operations(self, ops: &[(&str, usize)]) -> Self {
        for &(collection, total_files) in ops {
            self.indexing
                .start_operation(&CollectionId::from_name(collection), total_files);
        }
        self
    }

    /// Enable authentication with a given API key.
    pub fn with_auth(mut self, api_key: &str) -> Self {
        self.auth_config = AdminAuthConfig {
            enabled: true,
            header_name: "X-Admin-Key".to_owned(),
            api_key: Some(api_key.to_owned()),
        };
        self
    }

    /// Set a custom auth configuration.
    pub fn with_auth_config(mut self, config: AdminAuthConfig) -> Self {
        self.auth_config = config;
        self
    }

    /// Get shared reference to indexing operations (for pre-test setup before `build_client`).
    pub fn indexing(&self) -> &Arc<DefaultIndexingOperations> {
        &self.indexing
    }

    /// Build an `AdminState` without creating a Rocket client.
    pub fn build_state(&self) -> AdminState {
        let current_config = mcb_infrastructure::config::ConfigLoader::new()
            .load()
            .unwrap_or_else(|_| mcb_infrastructure::config::AppConfig::fallback());
        let metrics: Arc<dyn PerformanceMetricsInterface> =
            Arc::<AtomicPerformanceMetrics>::clone(&self.metrics);
        let indexing: Arc<dyn IndexingOperationsInterface> =
            Arc::<DefaultIndexingOperations>::clone(&self.indexing);
        AdminState {
            metrics,
            indexing,
            config_watcher: None,
            current_config,
            config_path: None,
            shutdown_coordinator: None,
            shutdown_timeout_secs: self.shutdown_timeout_secs,
            event_bus: TokioEventBusProvider::new_shared(),
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

    /// Build a Rocket test client, returning shared references to metrics and indexing.
    pub async fn build_client(
        self,
    ) -> (
        Client,
        Arc<AtomicPerformanceMetrics>,
        Arc<DefaultIndexingOperations>,
    ) {
        let metrics = Arc::clone(&self.metrics);
        let indexing = Arc::clone(&self.indexing);
        let state = self.build_state();
        let auth_config = Arc::new(self.auth_config);

        let client_result = Client::tracked(admin_rocket(state, auth_config, None)).await;
        assert!(client_result.is_ok(), "valid rocket instance");
        let client = match client_result {
            Ok(client) => client,
            Err(_) => unreachable!(),
        };

        (client, metrics, indexing)
    }
}

// ============================================================================
// Auth Constants (shared across auth tests)
// ============================================================================

/// Standard test API key used across auth integration tests.
pub const TEST_API_KEY: &str = "test-secret-key-12345";

/// Standard test auth header name.
pub const TEST_AUTH_HEADER: &str = "X-Admin-Key";

/// Create an `AdminAuthConfig` with authentication ENABLED and the standard test key.
pub fn create_auth_config() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: Some(TEST_API_KEY.to_owned()),
    }
}

/// Create an `AdminAuthConfig` with authentication ENABLED but NO key configured.
pub fn create_auth_config_no_key() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: None,
    }
}
