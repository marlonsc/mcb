//! Tests for Admin Web UI
//!
//! Tests the web dashboard pages and routes.

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
    assert!(html.contains("Entity Coverage"));
    assert!(html.contains("Domain Entities"));
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
async fn test_jobs_page_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/jobs").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(
        html.contains("Indexing Summary") || html.contains("Indexing") || html.contains("Jobs")
    );
}

#[rocket::async_test]
async fn test_removed_indexing_route_returns_not_found() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/indexing").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);
}

// ── Bulk Delete Tests ──────────────────────────────────────────────────────

#[rocket::async_test]
async fn test_entities_bulk_delete_redirects_to_list() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/ui/entities/organizations/bulk-delete")
        .header(rocket::http::ContentType::Form)
        .body("ids=id1,id2")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::SeeOther);
}

#[rocket::async_test]
async fn test_entities_bulk_delete_unknown_slug_returns_404() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/ui/entities/nonexistent/bulk-delete")
        .header(rocket::http::ContentType::Form)
        .body("ids=a,b")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
}

// ── LOV Endpoint Tests ─────────────────────────────────────────────────────

#[rocket::async_test]
async fn test_lov_endpoint_returns_json_array() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/lov/organizations").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    // Should be a JSON array (possibly empty since no adapter in web_rocket)
    assert!(
        body.starts_with('['),
        "LOV endpoint must return JSON array, got: {body}"
    );
}

#[rocket::async_test]
async fn test_lov_endpoint_unknown_slug_returns_404() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/lov/nonexistent").dispatch().await;

    assert_eq!(response.status(), Status::NotFound);
}

#[rocket::async_test]
async fn test_entities_index_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/entities").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Entities"));
    assert!(html.contains("Organizations"));
    assert!(html.contains("Users"));
}

#[rocket::async_test]
async fn test_entities_list_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/entities/organizations").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Organizations"));
    assert!(html.contains("org"));
    assert!(html.contains("Dashboard"));
    assert!(html.contains("Domain Entities"));
}

#[rocket::async_test]
async fn test_entities_list_unknown_slug_returns_404() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/entities/nonexistent").dispatch().await;

    assert_eq!(response.status(), Status::NotFound);
}

#[rocket::async_test]
async fn test_entities_new_form_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/entities/users/new").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("New Users"));
    assert!(html.contains("<form"));
    assert!(html.contains("Save"));
}

#[rocket::async_test]
async fn test_entities_new_form_unknown_slug_returns_404() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/entities/nonexistent/new").dispatch().await;

    assert_eq!(response.status(), Status::NotFound);
}

#[rocket::async_test]
async fn test_entities_detail_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/ui/entities/organizations/test-id-123")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Organizations"));
    assert!(html.contains("test-id-123"));
    assert!(html.contains("Edit"));
    assert!(html.contains("Delete"));
}

#[rocket::async_test]
async fn test_entities_detail_unknown_slug_returns_404() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/ui/entities/nonexistent/some-id")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
}

#[rocket::async_test]
async fn test_entities_edit_form_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/ui/entities/users/test-id-456/edit")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Edit Users"));
    assert!(html.contains("<form"));
    assert!(html.contains("PUT"));
}

#[rocket::async_test]
async fn test_entities_delete_confirm_returns_html() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .get("/ui/entities/plans/test-id-789/delete")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Delete Plans"));
    assert!(html.contains("test-id-789"));
    assert!(html.contains("Confirm Deletion"));
}

#[rocket::async_test]
async fn test_entities_new_form_has_enum_options() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/ui/entities/plans/new").dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let html = response.into_string().await.expect("response body");
    let html_lower = html.to_lowercase();
    assert!(html.contains("<select"));
    assert!(html_lower.contains("draft"));
    assert!(html_lower.contains("active"));
}

#[rocket::async_test]
async fn test_entities_create_redirects_to_list() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/ui/entities/organizations")
        .header(rocket::http::ContentType::Form)
        .body("name=TestOrg&slug=test-org")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::SeeOther);
}

#[rocket::async_test]
async fn test_entities_update_redirects_to_detail() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/ui/entities/organizations/test-id")
        .header(rocket::http::ContentType::Form)
        .body("name=Updated")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::SeeOther);
}

#[rocket::async_test]
async fn test_entities_delete_redirects_to_list() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/ui/entities/organizations/test-id/delete")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::SeeOther);
}

#[rocket::async_test]
async fn test_entities_create_unknown_slug_returns_404() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client
        .post("/ui/entities/nonexistent")
        .header(rocket::http::ContentType::Form)
        .body("name=Test")
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::NotFound);
}
