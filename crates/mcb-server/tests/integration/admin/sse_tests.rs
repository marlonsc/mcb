//! SSE Event Stream Tests
//!
//! Tests for the Server-Sent Events endpoint.

use mcb_domain::events::{DomainEvent, ServiceState};
use mcb_server::admin::sse::get_event_name;
use rstest::rstest;

#[rstest]
#[case(
    DomainEvent::ServiceStateChanged {
        name: "test-service".to_owned(),
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
        collection: "test-collection".to_owned(),
        total_files: 100,
    },
    "IndexingStarted"
)]
#[case(
    DomainEvent::IndexingProgress {
        collection: "test-collection".to_owned(),
        processed: 50,
        total: 100,
        current_file: Some("test.rs".to_owned()),
    },
    "IndexingProgress"
)]
#[case(
    DomainEvent::IndexingCompleted {
        collection: "test-collection".to_owned(),
        chunks: 500,
        duration_ms: 1000,
    },
    "IndexingCompleted"
)]
#[case(
    DomainEvent::ConfigReloaded {
        section: "embedding".to_owned(),
        timestamp: chrono::Utc::now().timestamp(),
    },
    "ConfigReloaded"
)]
#[case(
    DomainEvent::HealthCheckCompleted {
        status: "healthy".to_owned(),
        healthy_count: 3,
        unhealthy_count: 0,
    },
    "HealthCheckCompleted"
)]
#[case(
    DomainEvent::SearchExecuted {
        query: "test query".to_owned(),
        collection: "default".to_owned(),
        results: 10,
        duration_ms: 50,
    },
    "SearchExecuted"
)]
#[case(
    DomainEvent::CacheInvalidate {
        namespace: Some("embeddings".to_owned()),
    },
    "CacheInvalidate"
)]
#[case(
    DomainEvent::FileChangesDetected {
        root_path: "/project".to_owned(),
        added: 5,
        modified: 3,
        removed: 1,
    },
    "FileChangesDetected"
)]
#[case(
    DomainEvent::LogEvent {
        level: "WARN".to_owned(),
        message: "test warning".to_owned(),
        target: "mcb_server::test".to_owned(),
        timestamp: 1700000000000,
    },
    "LogEvent"
)]
#[case(
    DomainEvent::IndexRebuild {
        collection: Some("test-collection".to_owned()),
    },
    "IndexRebuild"
)]
#[case(
    DomainEvent::SyncCompleted {
        path: "/project".to_owned(),
        files_changed: 10,
    },
    "SyncCompleted"
)]
#[case(
    DomainEvent::SnapshotCreated {
        root_path: "/project".to_owned(),
        file_count: 42,
    },
    "SnapshotCreated"
)]
#[case(
    DomainEvent::ValidationStarted {
        operation_id: "op-1".to_owned(),
        workspace: "/project".to_owned(),
        validators: vec!["naming".to_owned()],
        total_files: 50,
    },
    "ValidationStarted"
)]
#[case(
    DomainEvent::ValidationProgress {
        operation_id: "op-1".to_owned(),
        processed: 25,
        total: 50,
        current_file: Some("lib.rs".to_owned()),
    },
    "ValidationProgress"
)]
#[case(
    DomainEvent::ValidationCompleted {
        operation_id: "op-1".to_owned(),
        workspace: "/project".to_owned(),
        total_violations: 3,
        errors: 1,
        warnings: 2,
        passed: false,
        duration_ms: 500,
    },
    "ValidationCompleted"
)]
#[test]
fn test_get_event_name_variants(#[case] event: DomainEvent, #[case] expected: &str) {
    assert_eq!(get_event_name(&event), expected);
}
