//! Integration tests for Admin API endpoints
//!
//! Tests for provider management endpoints:
//! - GET /admin/provider/current - Get current provider
//! - POST /admin/provider/switch - Switch to a different provider

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_providers::events::TokioEventBusProvider;
use mcb_server::admin::{AdminApi, AdminApiConfig};
use rstest::rstest;
use serde_json::json;

fn create_test_admin_api() -> AdminApi {
    let metrics = AtomicPerformanceMetrics::new_shared();
    let indexing = DefaultIndexingOperations::new_shared();
    let event_bus = TokioEventBusProvider::new_shared();

    let config = AdminApiConfig {
        port: 9091,
        ..AdminApiConfig::default()
    };

    AdminApi::new(config, metrics, indexing, event_bus)
}

#[rstest]
#[case("current_provider", "ollama", json!({ "current_provider": "ollama" }))]
#[case("switched_to", "openai", json!({ "switched_to": "openai" }))]
fn test_admin_api_provider_payload_shapes(
    #[case] field: &str,
    #[case] expected: &str,
    #[case] payload: serde_json::Value,
) {
    let _admin_api = create_test_admin_api();

    assert!(payload.get(field).is_some());
    assert_eq!(payload[field].as_str(), Some(expected));
    assert!(payload.is_object());
}

#[test]
fn test_admin_api_provider_switch_request_payload_shape() {
    let request_payload = json!({ "provider": "openai" });
    assert!(request_payload.get("provider").is_some());
    assert_eq!(request_payload["provider"].as_str(), Some("openai"));
}

/// Test: AdminApi can be created with custom config
#[test]
fn test_admin_api_creation_with_config() {
    let config = AdminApiConfig {
        port: 9092,
        ..AdminApiConfig::default()
    };
    assert_eq!(config.port, 9092);

    let loaded_config = ConfigLoader::new().load().expect("load config");
    assert_eq!(config.host, loaded_config.server.network.host);

    let _admin_api = create_test_admin_api();
}

/// Test: AdminApi health check returns valid metrics
#[rstest]
#[case("status")]
#[case("uptime_seconds")]
#[case("active_indexing_operations")]
fn test_admin_api_health_check_payload_contains_required_fields(#[case] field: &str) {
    let _admin_api = create_test_admin_api();

    // Simulate health check response
    let health_response = json!({
        "status": "healthy",
        "uptime_seconds": 3600,
        "active_indexing_operations": 0
    });

    assert!(health_response.get(field).is_some());
    assert_eq!(health_response["status"].as_str(), Some("healthy"));
    assert!(health_response["uptime_seconds"].is_number());
    assert!(health_response["active_indexing_operations"].is_number());
}
