//! SSE Event Stream Tests
//!
//! Tests for the Server-Sent Events endpoint.

use mcb_domain::events::{DomainEvent, ServiceState};
use mcb_server::admin::sse::get_event_name;
use rstest::rstest;

#[rstest]
#[case(
    DomainEvent::ServiceStateChanged {
        name: "test-service".to_string(),
        state: ServiceState::Running,
        previous_state: None,
    },
    "ServiceStateChanged"
)]
#[case(
    DomainEvent::MetricsSnapshot {
        timestamp: chrono::Utc::now().timestamp(),
    },
    "MetricsSnapshot"
)]
#[case(
    DomainEvent::IndexingStarted {
        collection: "test-collection".to_string(),
        total_files: 100,
    },
    "IndexingStarted"
)]
#[case(
    DomainEvent::IndexingProgress {
        collection: "test-collection".to_string(),
        processed: 50,
        total: 100,
        current_file: Some("test.rs".to_string()),
    },
    "IndexingProgress"
)]
#[case(
    DomainEvent::IndexingCompleted {
        collection: "test-collection".to_string(),
        chunks: 500,
        duration_ms: 1000,
    },
    "IndexingCompleted"
)]
#[case(
    DomainEvent::ConfigReloaded {
        section: "embedding".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    },
    "ConfigReloaded"
)]
#[case(
    DomainEvent::HealthCheckCompleted {
        status: "healthy".to_string(),
        healthy_count: 3,
        unhealthy_count: 0,
    },
    "HealthCheckCompleted"
)]
#[case(
    DomainEvent::SearchExecuted {
        query: "test query".to_string(),
        collection: "default".to_string(),
        results: 10,
        duration_ms: 50,
    },
    "SearchExecuted"
)]
#[case(
    DomainEvent::CacheInvalidate {
        namespace: Some("embeddings".to_string()),
    },
    "CacheInvalidate"
)]
#[case(
    DomainEvent::FileChangesDetected {
        root_path: "/project".to_string(),
        added: 5,
        modified: 3,
        removed: 1,
    },
    "FileChangesDetected"
)]
#[test]
fn test_get_event_name_variants(#[case] event: DomainEvent, #[case] expected: &str) {
    assert_eq!(get_event_name(&event), expected);
}
