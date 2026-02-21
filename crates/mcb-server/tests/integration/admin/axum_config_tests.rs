//! Axum config endpoint integration tests.
//!
//! Verifies GET /config, POST /config/reload, and PATCH /config/{section}
//! work correctly through the Axum router with AxumAdminAuth.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, patch, post};
use axum::{Extension, Router};
use http_body_util::BodyExt;
use mcb_infrastructure::config::{AppConfig, ConfigLoader};
use mcb_infrastructure::infrastructure::default_event_bus;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::admin::auth::{AdminAuthConfig, axum_admin_auth_layer};
use mcb_server::admin::config::handlers::{
    get_config_axum, reload_config_axum, update_config_section_axum,
};
use mcb_server::admin::handlers::AdminState;
use tower::ServiceExt;

use crate::utils::admin_harness::{TEST_API_KEY, TEST_AUTH_HEADER};

fn build_admin_state() -> Arc<AdminState> {
    let current_config = ConfigLoader::new()
        .load()
        .unwrap_or_else(|_| AppConfig::fallback());
    Arc::new(AdminState {
        metrics: AtomicPerformanceMetrics::new_shared(),
        indexing: DefaultIndexingOperations::new_shared(),
        config_watcher: None,
        current_config,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: default_event_bus(),
        service_manager: None,
        cache: None,
        project_workflow: None,
        vcs_entity: None,
        plan_entity: None,
        issue_entity: None,
        org_entity: None,
        tool_handlers: None,
    })
}

fn build_config_app(auth_config: AdminAuthConfig) -> Router {
    let admin_state = build_admin_state();
    Router::new()
        .route("/config", get(get_config_axum))
        .route("/config/reload", post(reload_config_axum))
        .route("/config/{section}", patch(update_config_section_axum))
        .layer(axum::middleware::from_fn(axum_admin_auth_layer))
        .layer(Extension(Arc::new(auth_config)))
        .with_state(admin_state)
}

fn auth_enabled() -> AdminAuthConfig {
    AdminAuthConfig {
        enabled: true,
        header_name: TEST_AUTH_HEADER.to_owned(),
        api_key: Some(TEST_API_KEY.to_owned()),
    }
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ---- GET /config ----

#[tokio::test]
async fn get_config_returns_200_with_valid_key() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/config")
                .header(TEST_AUTH_HEADER, TEST_API_KEY)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["success"], true);
    assert!(json["config"].is_object());
}

#[tokio::test]
async fn get_config_rejects_missing_key() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["error"], "missing_api_key");
}

#[tokio::test]
async fn get_config_rejects_wrong_key() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/config")
                .header(TEST_AUTH_HEADER, "wrong-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["error"], "invalid_api_key");
}

// ---- POST /config/reload ----

#[tokio::test]
async fn reload_config_returns_503_without_watcher() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/config/reload")
                .header(TEST_AUTH_HEADER, TEST_API_KEY)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["success"], false);
    assert!(json["message"].as_str().unwrap().contains("watcher"));
}

#[tokio::test]
async fn reload_config_rejects_missing_key() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/config/reload")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---- PATCH /config/{section} ----

#[tokio::test]
async fn update_config_invalid_section_returns_400() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config/nonexistent")
                .header(TEST_AUTH_HEADER, TEST_API_KEY)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"values":{"key":"val"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["success"], false);
    assert!(json["message"].as_str().unwrap().contains("Unknown"));
}

#[tokio::test]
async fn update_config_valid_section_no_watcher_returns_503() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config/server")
                .header(TEST_AUTH_HEADER, TEST_API_KEY)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"values":{"port":9090}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["success"], false);
}

#[tokio::test]
async fn update_config_rejects_missing_key() {
    let app = build_config_app(auth_enabled());
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/config/server")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"values":{"port":9090}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---- Auth disabled ----

#[tokio::test]
async fn get_config_allows_when_auth_disabled() {
    let app = build_config_app(AdminAuthConfig::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["success"], true);
}
