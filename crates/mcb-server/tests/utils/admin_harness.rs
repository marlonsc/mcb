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

use axum::Router;
use axum::body::Body;
use axum::http::StatusCode;
use axum::http::{Method, Request};
use http_body_util::BodyExt;
use mcb_domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::infrastructure::default_event_bus;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::admin::{auth::AdminAuthConfig, handlers::AdminState};
use mcb_server::transport::axum_http::{AppState, build_router};
use tower::ServiceExt;

// ============================================================================
// Admin Test Harness Builder
// ============================================================================

/// Builder for creating admin test state and Axum test client.
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

    /// Build an `AdminState` without creating a test client.
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
            event_bus: default_event_bus(),
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

        let app_state = Arc::new(AppState {
            metrics: Arc::clone(&metrics) as Arc<dyn PerformanceMetricsInterface>,
            indexing: Arc::clone(&indexing) as Arc<dyn IndexingOperationsInterface>,
            browser: None,
            browse_state: None,
            mcp_server: None,
            admin_state: Some(Arc::new(state)),
            auth_config: Some(auth_config),
        });
        let client = Client {
            app: build_router(&app_state),
        };

        (client, metrics, indexing)
    }
}

#[derive(Clone)]
pub struct Client {
    app: Router,
}

impl Client {
    #[must_use]
    pub fn get(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.app.clone(), Method::GET, path)
    }

    #[must_use]
    pub fn post(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.app.clone(), Method::POST, path)
    }

    #[must_use]
    pub fn patch(&self, path: &str) -> RequestBuilder {
        RequestBuilder::new(self.app.clone(), Method::PATCH, path)
    }
}

pub struct RequestBuilder {
    app: Router,
    method: Method,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl RequestBuilder {
    fn new(app: Router, method: Method, path: &str) -> Self {
        Self {
            app,
            method,
            path: path.to_owned(),
            headers: Vec::new(),
            body: None,
        }
    }

    #[must_use]
    pub fn header<T: IntoTestHeader>(mut self, header: T) -> Self {
        let (name, value) = header.into_test_header();
        self.headers.push((name, value));
        self
    }

    #[must_use]
    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_owned());
        self
    }

    pub async fn dispatch(self) -> TestResponse {
        let mut builder = Request::builder().method(self.method).uri(self.path);
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }

        let request = builder
            .body(Body::from(self.body.unwrap_or_default()))
            .unwrap_or_else(|_| unreachable!("valid test request"));

        let response = self
            .app
            .clone()
            .oneshot(request)
            .await
            .unwrap_or_else(|_| unreachable!("router should handle request"));

        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .unwrap_or_else(|_| unreachable!("collect response body"))
            .to_bytes();

        TestResponse {
            status,
            body: String::from_utf8(body.to_vec()).unwrap_or_default(),
        }
    }
}

pub trait IntoTestHeader {
    fn into_test_header(self) -> (String, String);
}

impl IntoTestHeader for (&str, &str) {
    fn into_test_header(self) -> (String, String) {
        (self.0.to_owned(), self.1.to_owned())
    }
}

pub struct TestResponse {
    status: StatusCode,
    body: String,
}

impl TestResponse {
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub async fn into_string(self) -> Option<String> {
        Some(self.body)
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
