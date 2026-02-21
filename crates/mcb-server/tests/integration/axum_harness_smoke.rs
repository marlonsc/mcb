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
