//! Admin HTTP/UI integration tests using the shared [`TestServer`].
//!
//! These tests exercise the production Axum route table (public assets,
//! protected admin UI/API, and auth middleware) against a real TCP-bound
//! server with an isolated database.

use crate::utils::test_server::{TestServer, assert_body_contains};
use anyhow::Result;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn public_root_redirects_to_ui() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };
    let response = server.get_no_redirect("/").await?;
    assert_eq!(response.status(), reqwest::StatusCode::TEMPORARY_REDIRECT);
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .ok_or_else(|| anyhow::anyhow!("location header missing"))?;
    assert_eq!(location, "/ui/");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn public_assets_served_without_auth() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let theme = server.get("/ui/theme.css").await?;
    assert_eq!(theme.status(), reqwest::StatusCode::OK);
    assert_eq!(
        theme
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .ok_or_else(|| anyhow::anyhow!("content-type header missing"))?,
        "text/css"
    );

    let shared = server.get("/ui/shared.js").await?;
    assert_eq!(shared.status(), reqwest::StatusCode::OK);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_ui_requires_authentication() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let dashboard = server.get("/ui/").await?;
    assert_eq!(dashboard.status(), reqwest::StatusCode::UNAUTHORIZED);

    let config = server.get("/ui/config").await?;
    assert_eq!(config.status(), reqwest::StatusCode::UNAUTHORIZED);

    let health = server.get("/ui/health").await?;
    assert_eq!(health.status(), reqwest::StatusCode::UNAUTHORIZED);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_api_requires_authentication() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let health = server.get("/health").await?;
    assert_eq!(health.status(), reqwest::StatusCode::UNAUTHORIZED);

    let jobs = server.get("/jobs").await?;
    assert_eq!(jobs.status(), reqwest::StatusCode::UNAUTHORIZED);

    let collections = server.get("/collections").await?;
    assert_eq!(collections.status(), reqwest::StatusCode::UNAUTHORIZED);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_ui_rejects_invalid_api_key() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let response = server
        .get_with_key("/ui/", "x-api-key", "invalid-key")
        .await?;
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_dashboard_authenticated_returns_html() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };
    let admin = server.seed_admin_user().await?;

    let response = server
        .get_with_key("/ui/", &admin.header_name, &admin.api_key)
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .ok_or_else(|| anyhow::anyhow!("content-type header missing"))?;
    assert!(content_type.as_bytes().starts_with(b"text/html"));
    assert_body_contains(response, "MCB Admin").await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_health_api_authenticated_returns_json() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };
    let admin = server.seed_admin_user().await?;

    let response = server
        .get_with_key("/health", &admin.header_name, &admin.api_key)
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    assert!(
        body["status"].as_str() == Some("healthy") || body["status"].as_str() == Some("degraded"),
        "unexpected health status: {body}"
    );
    assert!(body["embedding"]["provider"].is_string());
    assert!(body["embedding"]["dimensions"].is_number());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_jobs_api_authenticated_returns_structure() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };
    let admin = server.seed_admin_user().await?;

    let response = server
        .get_with_key("/jobs", &admin.header_name, &admin.api_key)
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    assert!(body["total"].is_number());
    assert!(body["running"].is_number());
    assert!(body["queued"].is_number());
    assert!(body["jobs"].is_array());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn admin_collections_api_authenticated_returns_array() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };
    let admin = server.seed_admin_user().await?;

    let response = server
        .get_with_key("/collections", &admin.header_name, &admin.api_key)
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await?;
    assert!(body.is_array(), "collections response must be an array");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn fallback_returns_404_page() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let response = server.get("/definitely-not-a-route").await?;
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    assert_body_contains(response, "Not Found").await?;
    Ok(())
}
