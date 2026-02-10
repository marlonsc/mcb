//! Admin Test Harness
//!
//! Shared test infrastructure for admin API tests. Eliminates duplication
//! of TestEventBus, AdminState construction, and fixture setup across
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

#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::infrastructure::{DomainEventStream, EventBusProvider};
use mcb_domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::admin::{auth::AdminAuthConfig, handlers::AdminState, routes::admin_rocket};
use rocket::local::asynchronous::Client;
use serde::Deserialize;

// ============================================================================
// Shared Test Event Bus
// ============================================================================

/// Null EventBus for admin testing.
pub struct TestEventBus;

#[async_trait]
impl EventBusProvider for TestEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        Ok(Box::pin(futures::stream::empty()))
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

// ============================================================================
// Admin Test Harness Builder
// ============================================================================

/// Builder for creating admin test state and Rocket client.
///
/// Provides a fluent API for configuring AdminState with metrics,
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

    /// Record query metrics: each tuple is (latency_ms, success, cache_hit).
    pub fn with_recorded_metrics(self, queries: &[(u64, bool, bool)], connections: i64) -> Self {
        for &(latency_ms, success, cache_hit) in queries {
            self.metrics.record_query(latency_ms, success, cache_hit);
        }
        self.metrics.update_active_connections(connections);
        self
    }

    /// Start indexing operations: each tuple is (collection_name, total_files).
    pub fn with_indexing_operations(self, ops: &[(&str, usize)]) -> Self {
        for &(collection, total_files) in ops {
            self.indexing
                .start_operation(&CollectionId::new(collection), total_files);
        }
        self
    }

    /// Enable authentication with a given API key.
    pub fn with_auth(mut self, api_key: &str) -> Self {
        self.auth_config = AdminAuthConfig {
            enabled: true,
            header_name: "X-Admin-Key".to_string(),
            api_key: Some(api_key.to_string()),
        };
        self
    }

    /// Set a custom auth configuration.
    pub fn with_auth_config(mut self, config: AdminAuthConfig) -> Self {
        self.auth_config = config;
        self
    }

    /// Get shared reference to metrics (for pre-test recording or post-test assertion).
    pub fn metrics(&self) -> &Arc<AtomicPerformanceMetrics> {
        &self.metrics
    }

    /// Get shared reference to indexing operations.
    pub fn indexing(&self) -> &Arc<DefaultIndexingOperations> {
        &self.indexing
    }

    /// Build an AdminState without creating a Rocket client.
    pub fn build_state(&self) -> AdminState {
        AdminState {
            metrics: self.metrics.clone(),
            indexing: self.indexing.clone(),
            config_watcher: None,
            current_config: mcb_infrastructure::config::types::AppConfig::default(),
            config_path: None,
            shutdown_coordinator: None,
            shutdown_timeout_secs: self.shutdown_timeout_secs,
            event_bus: Arc::new(TestEventBus),
            service_manager: None,
            cache: None,
            project_workflow: None,
            vcs_entity: None,
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
        let metrics = self.metrics.clone();
        let indexing = self.indexing.clone();
        let state = self.build_state();
        let auth_config = Arc::new(self.auth_config);

        let client = Client::tracked(admin_rocket(state, auth_config, None))
            .await
            .expect("valid rocket instance");

        (client, metrics, indexing)
    }
}

pub fn create_test_state() -> AdminState {
    AdminTestHarness::new().build_state()
}

// ============================================================================
// JSON Fixture Loading
// ============================================================================

/// A single recorded query for fixture loading.
#[derive(Debug, Deserialize)]
pub struct FixtureQuery {
    pub latency_ms: u64,
    pub success: bool,
    pub cache_hit: bool,
}

/// Metrics scenario loaded from JSON fixture.
#[derive(Debug, Deserialize)]
pub struct MetricsScenario {
    pub queries: Vec<FixtureQuery>,
    pub active_connections: i64,
}

/// Indexing operation loaded from JSON fixture.
#[derive(Debug, Deserialize)]
pub struct FixtureOperation {
    pub collection: String,
    pub total_files: usize,
    #[serde(default)]
    pub processed_files: usize,
    #[serde(default)]
    pub current_file: Option<String>,
}

/// Indexing scenario loaded from JSON fixture.
#[derive(Debug, Deserialize)]
pub struct IndexingScenario {
    pub operations: Vec<FixtureOperation>,
}

/// Top-level fixture file structure.
#[derive(Debug, Deserialize)]
pub struct AdminFixtures {
    pub metrics: std::collections::HashMap<String, MetricsScenario>,
    pub indexing: std::collections::HashMap<String, IndexingScenario>,
}

impl AdminFixtures {
    /// Load fixtures from the standard fixture file location.
    pub fn load() -> Self {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/admin_scenarios.json");
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "Failed to load admin fixtures from {}: {}",
                path.display(),
                e
            )
        });
        serde_json::from_str(&content).expect("Failed to parse admin fixtures JSON")
    }

    /// Apply a named metrics scenario to a harness.
    pub fn apply_metrics(&self, name: &str, harness: AdminTestHarness) -> AdminTestHarness {
        let scenario = self
            .metrics
            .get(name)
            .unwrap_or_else(|| panic!("Unknown metrics scenario: {}", name));

        let queries: Vec<(u64, bool, bool)> = scenario
            .queries
            .iter()
            .map(|q| (q.latency_ms, q.success, q.cache_hit))
            .collect();

        harness.with_recorded_metrics(&queries, scenario.active_connections)
    }

    /// Apply a named indexing scenario to a harness.
    pub fn apply_indexing(&self, name: &str, harness: AdminTestHarness) -> AdminTestHarness {
        let scenario = self
            .indexing
            .get(name)
            .unwrap_or_else(|| panic!("Unknown indexing scenario: {}", name));

        for op in &scenario.operations {
            let op_id = harness
                .indexing()
                .start_operation(&CollectionId::new(&op.collection), op.total_files);

            if op.processed_files > 0 || op.current_file.is_some() {
                harness.indexing().update_progress(
                    &op_id,
                    op.current_file.clone(),
                    op.processed_files,
                );
            }
        }

        harness
    }
}

// ============================================================================
// Pre-built Fixture Scenarios (Rust-native, no JSON needed)
// ============================================================================

/// High-traffic scenario: 10 queries with mixed results, 42 active connections.
///
/// Metrics: 8 successful, 2 failed, 5 cache hits, 5 misses.
/// Average latency: ~141.5ms. Cache hit rate: 50%.
pub fn high_traffic_fixture() -> AdminTestHarness {
    AdminTestHarness::new().with_recorded_metrics(
        &[
            (50, true, true),
            (75, true, true),
            (120, true, false),
            (200, true, false),
            (90, true, true),
            (300, false, false),
            (150, true, false),
            (80, true, true),
            (250, false, false),
            (100, true, true),
        ],
        42,
    )
}

/// Active indexing: 3 concurrent operations at various stages.
pub fn active_indexing_fixture() -> AdminTestHarness {
    AdminTestHarness::new().with_indexing_operations(&[
        ("project-alpha", 100),
        ("project-beta", 200),
        ("project-gamma", 50),
    ])
}

/// Full-stack scenario: metrics + indexing + authentication enabled.
pub fn full_stack_fixture() -> AdminTestHarness {
    high_traffic_fixture()
        .with_indexing_operations(&[("main-repo", 500)])
        .with_auth("admin-secret-key")
}

// ============================================================================
// Auth Constants (shared across auth tests)
// ============================================================================

/// Standard test API key used across auth integration tests.
pub const TEST_API_KEY: &str = "test-secret-key-12345";

/// Standard test auth header name.
pub const TEST_AUTH_HEADER: &str = "X-Admin-Key";

/// Create an AdminAuthConfig with authentication ENABLED and the standard test key.
pub fn create_auth_config() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_string(),
        api_key: Some(TEST_API_KEY.to_string()),
    }
}

/// Create an AdminAuthConfig with authentication ENABLED but NO key configured.
pub fn create_auth_config_no_key() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_string(),
        api_key: None,
    }
}
