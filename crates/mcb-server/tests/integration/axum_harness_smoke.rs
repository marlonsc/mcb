use axum::http::StatusCode;

use crate::utils::axum_harness::{test_app, test_get, test_get_auth, test_post};

#[tokio::test]
async fn axum_test_smoke_health_returns_200() {
    let app = test_app();
    let resp = test_get(&app, "/health").await;

    assert_eq!(resp.status, StatusCode::OK);

    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn axum_health_returns_admin_format() {
    let app = test_app();
    let resp = test_get(&app, "/health").await;

    assert_eq!(resp.status, StatusCode::OK);

    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "healthy");
    assert!(
        body["uptime_seconds"].is_number(),
        "should include uptime_seconds"
    );
    assert!(
        body["active_indexing_operations"].is_number(),
        "should include active_indexing_operations"
    );
}

#[tokio::test]
async fn axum_ready_returns_valid_status() {
    let app = test_app();
    let resp = test_get(&app, "/ready").await;

    let body: serde_json::Value = resp.json();
    assert!(body["ready"].is_boolean(), "should include ready field");
    assert!(
        body["uptime_seconds"].is_number(),
        "should include uptime_seconds"
    );

    assert!(
        resp.status == StatusCode::OK || resp.status == StatusCode::SERVICE_UNAVAILABLE,
        "ready should return 200 or 503, got {}",
        resp.status
    );
}

#[tokio::test]
async fn axum_ready_response_structure() {
    let app = test_app();
    let resp = test_get(&app, "/ready").await;

    let body: serde_json::Value = resp.json();
    assert!(body.get("ready").is_some(), "missing 'ready' field");
    assert!(
        body.get("uptime_seconds").is_some(),
        "missing 'uptime_seconds' field"
    );
}

#[tokio::test]
async fn axum_test_smoke_unknown_route_returns_404() {
    let app = test_app();
    let resp = test_get(&app, "/nonexistent").await;

    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn axum_test_smoke_post_to_health_returns_405() {
    let app = test_app();
    let resp = test_post(&app, "/health", "{}").await;

    assert_eq!(resp.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn axum_test_smoke_auth_header_passthrough() {
    let app = test_app();
    let resp = test_get_auth(&app, "/health", "test-key-123").await;

    assert_eq!(resp.status, StatusCode::OK);
}
