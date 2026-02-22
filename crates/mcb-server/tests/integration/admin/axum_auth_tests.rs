//! Axum admin auth extractor integration tests.

use std::sync::Arc;

use axum::routing::get;
use axum::{Extension, Router};
use mcb_server::admin::auth::{AdminAuthConfig, AxumAdminAuth};
use tower::ServiceExt;

use crate::utils::admin_harness::{TEST_API_KEY, TEST_AUTH_HEADER};

async fn protected_handler(_auth: AxumAdminAuth) -> &'static str {
    "ok"
}

fn build_axum_app(config: AdminAuthConfig) -> Router {
    Router::new()
        .route("/protected", get(protected_handler))
        .layer(axum::middleware::from_fn(
            mcb_server::admin::auth::axum_admin_auth_layer,
        ))
        .layer(Extension(Arc::new(config)))
}

#[tokio::test]
async fn axum_auth_reject_missing_key() {
    let app = build_axum_app(AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: Some(TEST_API_KEY.to_owned()),
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/protected")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "missing_api_key");
    assert!(json["message"].as_str().unwrap().contains("X-Admin-Key"));
}

#[tokio::test]
async fn axum_auth_reject_wrong_key() {
    let app = build_axum_app(AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: Some(TEST_API_KEY.to_owned()),
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/protected")
                .header(TEST_AUTH_HEADER, "wrong-key")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "invalid_api_key");
}

#[tokio::test]
async fn axum_auth_accept_valid_key() {
    let app = build_axum_app(AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: Some(TEST_API_KEY.to_owned()),
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/protected")
                .header(TEST_AUTH_HEADER, TEST_API_KEY)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn axum_auth_disabled_allows_all() {
    let app = build_axum_app(AdminAuthConfig::default());

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/protected")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn axum_auth_not_configured_returns_503() {
    let app = build_axum_app(AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: None,
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/protected")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "auth_not_configured");
}
