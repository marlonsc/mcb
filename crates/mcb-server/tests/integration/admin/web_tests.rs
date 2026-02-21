//! Tests for Admin Web UI
//!
//! Tests the web dashboard pages and routes using Axum's in-process dispatch.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mcb_server::admin::web::web_router;
use tower::ServiceExt;

async fn get(path: &str) -> (StatusCode, String) {
    let app = web_router();
    let req = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("valid request");
    let resp = app.oneshot(req).await.expect("router handles request");
    let status = resp.status();
    let body = resp
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    (status, String::from_utf8(body.to_vec()).expect("utf-8"))
}

async fn post_form(path: &str, form_body: &str) -> StatusCode {
    let app = web_router();
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_body.to_owned()))
        .expect("valid request");
    let resp = app.oneshot(req).await.expect("router handles request");
    resp.status()
}

#[tokio::test]
async fn test_dashboard_returns_html() {
    let (status, html) = get("/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Dashboard"));
    assert!(html.contains("Entity Coverage"));
    assert!(html.contains("Domain Entities"));
}

#[tokio::test]
async fn test_config_page_returns_html() {
    let (status, html) = get("/ui/config").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Configuration"));
}

#[tokio::test]
async fn test_health_page_returns_html() {
    let (status, html) = get("/ui/health").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Health Status"));
}

#[tokio::test]
async fn test_jobs_page_returns_html() {
    let (status, html) = get("/ui/jobs").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        html.contains("Indexing Summary") || html.contains("Indexing") || html.contains("Jobs")
    );
}

#[tokio::test]
async fn test_removed_indexing_route_returns_not_found() {
    let (status, _) = get("/ui/indexing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_entities_bulk_delete_redirects_to_list() {
    let status = post_form("/ui/entities/organizations/bulk-delete", "ids=id1,id2").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_entities_bulk_delete_unknown_slug_returns_404() {
    let status = post_form("/ui/entities/nonexistent/bulk-delete", "ids=a,b").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_lov_endpoint_returns_json_array() {
    let (status, body) = get("/ui/lov/organizations").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.starts_with('['),
        "LOV endpoint must return JSON array, got: {body}"
    );
}

#[tokio::test]
async fn test_lov_endpoint_unknown_slug_returns_404() {
    let (status, _) = get("/ui/lov/nonexistent").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_entities_index_returns_html() {
    let (status, html) = get("/ui/entities").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Entities"));
    assert!(html.contains("Organizations"));
    assert!(html.contains("Users"));
}

#[tokio::test]
async fn test_entities_list_returns_html() {
    let (status, html) = get("/ui/entities/organizations").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Organizations"));
    assert!(html.contains("org"));
    assert!(html.contains("Dashboard"));
    assert!(html.contains("Domain Entities"));
}

#[tokio::test]
async fn test_agent_sessions_list_returns_html() {
    let (status, html) = get("/ui/entities/agent-sessions").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Agent Sessions"));
}

#[tokio::test]
async fn test_delegations_list_returns_html() {
    let (status, html) = get("/ui/entities/delegations").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Delegations"));
}

#[tokio::test]
async fn test_tool_calls_list_returns_html() {
    let (status, html) = get("/ui/entities/tool-calls").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Tool Calls"));
}

#[tokio::test]
async fn test_checkpoints_list_returns_html() {
    let (status, html) = get("/ui/entities/checkpoints").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Checkpoints"));
}

#[tokio::test]
async fn test_entities_list_unknown_slug_returns_404() {
    let (status, _) = get("/ui/entities/nonexistent").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_entities_new_form_returns_html() {
    let (status, html) = get("/ui/entities/users/new").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("New Users"));
    assert!(html.contains("<form"));
    assert!(html.contains("Save"));
}

#[tokio::test]
async fn test_entities_new_form_unknown_slug_returns_404() {
    let (status, _) = get("/ui/entities/nonexistent/new").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_entities_detail_returns_html() {
    let (status, html) = get("/ui/entities/organizations/test-id-123").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Organizations"));
    assert!(html.contains("test-id-123"));
    assert!(html.contains("Edit"));
    assert!(html.contains("Delete"));
}

#[tokio::test]
async fn test_entities_detail_unknown_slug_returns_404() {
    let (status, _) = get("/ui/entities/nonexistent/some-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_entities_edit_form_returns_html() {
    let (status, html) = get("/ui/entities/users/test-id-456/edit").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Edit Users"));
    assert!(html.contains("<form"));
    assert!(html.contains("PUT"));
}

#[tokio::test]
async fn test_entities_delete_confirm_returns_html() {
    let (status, html) = get("/ui/entities/plans/test-id-789/delete").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Delete Plans"));
    assert!(html.contains("test-id-789"));
    assert!(html.contains("Confirm Deletion"));
}

#[tokio::test]
async fn test_entities_new_form_has_enum_options() {
    let (status, html) = get("/ui/entities/plans/new").await;
    assert_eq!(status, StatusCode::OK);
    let html_lower = html.to_lowercase();
    assert!(html.contains("<select"));
    assert!(html_lower.contains("draft"));
    assert!(html_lower.contains("active"));
}

#[tokio::test]
async fn test_entities_create_redirects_to_list() {
    let status = post_form("/ui/entities/organizations", "name=TestOrg&slug=test-org").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_entities_update_redirects_to_detail() {
    let status = post_form("/ui/entities/organizations/test-id", "name=Updated").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_entities_delete_redirects_to_list() {
    let status = post_form("/ui/entities/organizations/test-id/delete", "").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_entities_create_unknown_slug_returns_404() {
    let status = post_form("/ui/entities/nonexistent", "name=Test").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
