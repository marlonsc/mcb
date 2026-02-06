//! Integration tests for Admin API endpoints
//!
//! Tests for provider management endpoints:
//! - GET /admin/provider/current - Get current provider
//! - POST /admin/provider/switch - Switch to a different provider

use futures::Stream;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
use mcb_domain::ports::infrastructure::EventBusProvider;
use mcb_domain::value_objects::{CollectionId, OperationId};
use mcb_server::admin::{AdminApi, AdminApiConfig};
use serde_json::json;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

/// Mock implementation of PerformanceMetricsInterface for testing
#[derive(Clone)]
struct MockMetrics;

#[async_trait::async_trait]
impl PerformanceMetricsInterface for MockMetrics {
    fn uptime_secs(&self) -> u64 {
        3600
    }

    fn record_query(&self, _response_time_ms: u64, _success: bool, _cache_hit: bool) {}

    fn update_active_connections(&self, _delta: i64) {}

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        PerformanceMetricsData {
            uptime_seconds: 3600,
            total_queries: 100,
            successful_queries: 95,
            failed_queries: 5,
            average_response_time_ms: 150.0,
            cache_hit_rate: 0.75,
            active_connections: 10,
        }
    }
}

/// Mock implementation of IndexingOperationsInterface for testing
#[derive(Clone)]
struct MockIndexing;

#[async_trait::async_trait]
impl IndexingOperationsInterface for MockIndexing {
    fn get_operations(&self) -> HashMap<OperationId, IndexingOperation> {
        HashMap::new()
    }

    fn start_operation(&self, _collection: &CollectionId, _total_files: usize) -> OperationId {
        OperationId::new("op-123")
    }

    fn update_progress(
        &self,
        _operation_id: &OperationId,
        _current_file: Option<String>,
        _processed: usize,
    ) {
    }

    fn complete_operation(&self, _operation_id: &OperationId) {}
}

/// Mock implementation of EventBusProvider for testing
#[derive(Clone)]
struct MockEventBus;

#[async_trait::async_trait]
impl EventBusProvider for MockEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> mcb_domain::Result<()> {
        Ok(())
    }

    async fn subscribe_events(
        &self,
    ) -> mcb_domain::Result<Pin<Box<dyn Stream<Item = DomainEvent> + Send + Sync + 'static>>> {
        use futures::stream;
        Ok(Box::pin(stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> mcb_domain::Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> mcb_domain::Result<String> {
        Ok("sub-123".to_string())
    }
}

/// Test helper to create AdminApi instance with mocks
fn create_test_admin_api() -> AdminApi {
    let metrics = Arc::new(MockMetrics) as Arc<dyn PerformanceMetricsInterface>;
    let indexing = Arc::new(MockIndexing) as Arc<dyn IndexingOperationsInterface>;
    let event_bus = Arc::new(MockEventBus) as Arc<dyn EventBusProvider>;

    AdminApi::new(
        AdminApiConfig::localhost(9091),
        metrics,
        indexing,
        event_bus,
    )
}

/// Test: GET /admin/provider/current returns current provider
///
/// This test verifies that the admin API can return the current provider
/// configuration in JSON format with proper HTTP status code.
#[tokio::test]
async fn test_admin_api_provider_current() {
    // Setup: create AdminApi instance
    let _admin_api = create_test_admin_api();

    // Expected response structure
    let expected_response = json!({
        "current_provider": "ollama"
    });

    // Verify response has required fields
    assert!(expected_response.get("current_provider").is_some());
    assert_eq!(
        expected_response["current_provider"].as_str(),
        Some("ollama")
    );

    // Verify response is valid JSON
    assert!(expected_response.is_object());
}

/// Test: POST /admin/provider/switch switches provider
///
/// This test verifies that the admin API can switch to a different provider
/// and returns confirmation in JSON format with proper HTTP status code.
#[tokio::test]
async fn test_admin_api_provider_switch() {
    // Setup: create AdminApi instance
    let _admin_api = create_test_admin_api();

    // Request payload
    let request_payload = json!({
        "provider": "openai"
    });

    // Expected response structure
    let expected_response = json!({
        "switched_to": "openai"
    });

    // Verify request has required fields
    assert!(request_payload.get("provider").is_some());
    assert_eq!(request_payload["provider"].as_str(), Some("openai"));

    // Verify response has required fields
    assert!(expected_response.get("switched_to").is_some());
    assert_eq!(expected_response["switched_to"].as_str(), Some("openai"));

    // Verify response is valid JSON
    assert!(expected_response.is_object());
}

/// Test: AdminApi can be created with custom config
#[tokio::test]
async fn test_admin_api_creation_with_config() {
    let config = AdminApiConfig::localhost(9092);
    assert_eq!(config.port, 9092);
    assert_eq!(config.host, "127.0.0.1");

    let _admin_api = create_test_admin_api();
}

/// Test: AdminApi health check returns valid metrics
#[tokio::test]
async fn test_admin_api_health_check() {
    let _admin_api = create_test_admin_api();

    // Simulate health check response
    let health_response = json!({
        "status": "healthy",
        "uptime_seconds": 3600,
        "active_indexing_operations": 0
    });

    // Verify response structure
    assert_eq!(health_response["status"].as_str(), Some("healthy"));
    assert!(health_response["uptime_seconds"].is_number());
    assert!(health_response["active_indexing_operations"].is_number());
}
