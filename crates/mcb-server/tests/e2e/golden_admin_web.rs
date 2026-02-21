//! Golden E2E tests for Admin Web UI
//!
//! Verifies that the admin web UI is accessible via the production path:
//! the Axum web router (`web_router_with_state`) that serves all /ui/* and static assets.
//! SSOT: admin UI lives in Axum; these tests use the same router as production.

#![cfg(test)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mcb_server::admin::web::router::web_router_with_state;
use tower::ServiceExt;

use crate::utils::admin_harness::AdminTestHarness;

fn app() -> axum::Router {
    web_router_with_state(AdminTestHarness::new().build_state())
}

async fn get(path: &str) -> (StatusCode, String, Option<String>) {
    let app = app();
    let req = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("valid request");
    let resp = app.oneshot(req).await.expect("router handles request");
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let body = resp
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let text = String::from_utf8(body.to_vec()).expect("utf-8");
    (status, text, content_type)
}

async fn post_form(path: &str, body: &str) -> StatusCode {
    let app = app();
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .expect("valid request");
    let resp = app.oneshot(req).await.expect("router handles request");
    resp.status()
}

#[tokio::test]
async fn test_admin_rocket_dashboard_is_accessible() {
    let (status, html, _) = get("/").await;
    assert_eq!(status, StatusCode::OK, "Dashboard (/) must return 200 OK");
    assert!(
        html.contains("<!DOCTYPE html>"),
        "Dashboard must return HTML"
    );
    assert!(
        html.contains("Dashboard") || html.contains("MCB"),
        "Dashboard must contain dashboard content"
    );
}

#[tokio::test]
async fn test_admin_rocket_config_page_is_accessible() {
    let (status, _, _) = get("/ui/config").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_health_page_is_accessible() {
    let (status, _, _) = get("/ui/health").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_removed_indexing_route_returns_not_found() {
    let (status, _, _) = get("/ui/indexing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_rocket_jobs_page_is_accessible() {
    let (status, _, _) = get("/ui/jobs").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_browse_page_is_accessible() {
    let (status, _, _) = get("/ui/browse").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_favicon_is_accessible() {
    let (status, _, content_type) = get("/favicon.ico").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        content_type
            .as_ref()
            .map_or(false, |ct| ct.starts_with("image/svg+xml")),
        "Unexpected Content-Type: {content_type:?}"
    );
}

#[tokio::test]
async fn test_admin_rocket_theme_css_is_accessible() {
    let (status, _, content_type) = get("/ui/theme.css").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        content_type
            .as_ref()
            .map_or(false, |ct| ct.starts_with("text/css")),
        "Unexpected Content-Type: {content_type:?}"
    );
}

#[tokio::test]
async fn test_admin_rocket_shared_js_is_accessible() {
    let (status, _, content_type) = get("/ui/shared.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        content_type
            .as_ref()
            .map_or(false, |ct| ct.contains("javascript")),
        "Unexpected Content-Type for /ui/shared.js: {content_type:?}"
    );
}

#[tokio::test]
async fn test_admin_rocket_entities_bulk_delete_is_accessible() {
    let status = post_form("/ui/entities/organizations/bulk-delete", "ids=a,b").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_admin_rocket_lov_endpoint_is_accessible() {
    let (status, body, _) = get("/ui/lov/organizations").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.starts_with('['), "LOV endpoint must return JSON array");
}

#[tokio::test]
async fn test_admin_rocket_agent_sessions_page_is_accessible() {
    let (status, _, _) = get("/ui/entities/agent-sessions").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_delegations_page_is_accessible() {
    let (status, _, _) = get("/ui/entities/delegations").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_tool_calls_page_is_accessible() {
    let (status, _, _) = get("/ui/entities/tool-calls").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_rocket_checkpoints_page_is_accessible() {
    let (status, _, _) = get("/ui/entities/checkpoints").await;
    assert_eq!(status, StatusCode::OK);
}
