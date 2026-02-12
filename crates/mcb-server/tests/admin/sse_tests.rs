//! SSE Event Stream Tests
//!
//! Tests for the Server-Sent Events endpoint.

use mcb_domain::events::{DomainEvent, ServiceState};
use mcb_server::admin::sse::get_event_name;

#[test]
fn test_get_event_name_service_state_changed() {
    let event = DomainEvent::ServiceStateChanged {
        name: "test-service".to_string(),
        state: ServiceState::Running,
        previous_state: None,
    };
    assert_eq!(get_event_name(&event), "ServiceStateChanged");
}

#[test]
fn test_get_event_name_metrics_snapshot() {
    let event = DomainEvent::MetricsSnapshot {
        timestamp: chrono::Utc::now().timestamp(),
    };
    assert_eq!(get_event_name(&event), "MetricsSnapshot");
}

#[test]
fn test_get_event_name_indexing_started() {
    let event = DomainEvent::IndexingStarted {
        collection: "test-collection".to_string(),
        total_files: 100,
    };
    assert_eq!(get_event_name(&event), "IndexingStarted");
}

#[test]
fn test_get_event_name_indexing_progress() {
    let event = DomainEvent::IndexingProgress {
        collection: "test-collection".to_string(),
        processed: 50,
        total: 100,
        current_file: Some("test.rs".to_string()),
    };
    assert_eq!(get_event_name(&event), "IndexingProgress");
}

#[test]
fn test_get_event_name_indexing_completed() {
    let event = DomainEvent::IndexingCompleted {
        collection: "test-collection".to_string(),
        chunks: 500,
        duration_ms: 1000,
    };
    assert_eq!(get_event_name(&event), "IndexingCompleted");
}

#[test]
fn test_get_event_name_config_reloaded() {
    let event = DomainEvent::ConfigReloaded {
        section: "embedding".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    assert_eq!(get_event_name(&event), "ConfigReloaded");
}

#[test]
fn test_get_event_name_health_check_completed() {
    let event = DomainEvent::HealthCheckCompleted {
        status: "healthy".to_string(),
        healthy_count: 3,
        unhealthy_count: 0,
    };
    assert_eq!(get_event_name(&event), "HealthCheckCompleted");
}

#[test]
fn test_get_event_name_search_executed() {
    let event = DomainEvent::SearchExecuted {
        query: "test query".to_string(),
        collection: "default".to_string(),
        results: 10,
        duration_ms: 50,
    };
    assert_eq!(get_event_name(&event), "SearchExecuted");
}

#[test]
fn test_get_event_name_cache_invalidate() {
    let event = DomainEvent::CacheInvalidate {
        namespace: Some("embeddings".to_string()),
    };
    assert_eq!(get_event_name(&event), "CacheInvalidate");
}

#[test]
fn test_get_event_name_file_changes_detected() {
    let event = DomainEvent::FileChangesDetected {
        root_path: "/project".to_string(),
        added: 5,
        modified: 3,
        removed: 1,
    };
    assert_eq!(get_event_name(&event), "FileChangesDetected");
}
