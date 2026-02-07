//! Tests for Admin Web UI
//!
//! Tests the web dashboard pages and routes.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use mcb_server::admin::web::web_rocket;
use rocket::http::Status;
use rocket::local::asynchronous::Client;

#[rocket::async_test]
async fn test_dashboard_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Dashboard"));
}

#[rocket::async_test]
async fn test_config_page_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/config").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("Configuration"));
}

#[rocket::async_test]
async fn test_health_page_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/health").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("Health Status"));
}

#[rocket::async_test]
async fn test_indexing_page_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/indexing").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("Indexing Status"));
}

#[rocket::async_test]
async fn test_favicon_returns_svg() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/favicon.ico").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.content_type().map(|ct| ct.to_string()),
        Some("image/svg+xml".to_string())
    );
}
