//! Golden E2E tests for Admin Web UI
//!
//! CRITICAL: These tests verify that the admin web UI is actually accessible
//! in the REAL production server configuration (admin_rocket), not just in
//! isolated web_rocket tests.
//!
//! WHY THIS EXISTS: v0.2.0 shipped with broken admin UI (404 on all routes)
//! because web routes were only mounted in web_rocket() but NOT in admin_rocket()
//! which is what the production server actually uses.

use crate::admin::harness::AdminTestHarness;
use rocket::http::Status;

/// Test that the admin dashboard is accessible via the REAL admin_rocket instance
///
/// This is the CRITICAL test that should have caught the v0.2.0 bug where
/// admin web UI returned 404 because routes were not mounted in admin_rocket.
#[rocket::async_test]
async fn test_admin_rocket_dashboard_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/").dispatch().await;

    assert_eq!(
        response.status(),
        Status::Ok,
        "Dashboard (/) must return 200 OK, not 404. This is the PRODUCTION route."
    );

    let html = response.into_string().await.expect("response body");
    assert!(
        html.contains("<!DOCTYPE html>"),
        "Dashboard must return HTML"
    );
    assert!(
        html.contains("Dashboard") || html.contains("MCB"),
        "Dashboard must contain dashboard content"
    );
}

/// Test that /ui/config is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_config_page_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/config").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}

/// Test that /ui/health is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_health_page_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/health").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}

/// Test that legacy /ui/indexing route is removed (use /ui/jobs instead)
#[rocket::async_test]
async fn test_admin_rocket_legacy_indexing_route_removed() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/indexing").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);
}

/// Test that /ui/jobs is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_jobs_page_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/jobs").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}

/// Test that /ui/browse is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_browse_page_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/browse").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}

/// Test that /favicon.ico is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_favicon_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/favicon.ico").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.content_type().map(|ct| ct.to_string()),
        Some("image/svg+xml".to_string())
    );
}

/// Test that theme CSS is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_theme_css_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/theme.css").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.content_type().map(|ct| ct.to_string()),
        Some("text/css; charset=utf-8".to_string())
    );
}

/// Test that shared JS is accessible via admin_rocket
#[rocket::async_test]
async fn test_admin_rocket_shared_js_is_accessible() {
    let (client, _, _) = AdminTestHarness::new().build_client().await;

    let response = client.get("/ui/shared.js").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let content_type = response.content_type().map(|ct| ct.to_string());
    assert!(
        matches!(
            content_type.as_deref(),
            Some("text/javascript") | Some("text/javascript; charset=utf-8")
        ),
        "Unexpected Content-Type for /ui/shared.js: {:?}",
        content_type
    );
}
