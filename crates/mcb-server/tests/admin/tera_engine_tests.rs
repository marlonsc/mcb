use mcb_server::admin::web::web_rocket;
use rocket::http::Status;
use rocket::local::asynchronous::Client;

#[rocket::async_test]
async fn test_tera_base_layout_renders_with_hbs_pages() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let html = response.into_string().await.expect("response body");
    assert!(
        html.contains("<!DOCTYPE html>"),
        "Dashboard still renders after Template::custom() switch"
    );
    assert!(html.contains("Memory Context Browser"));
}

#[rocket::async_test]
async fn test_tera_engine_coexists_with_handlebars() {
    let client = Client::tracked(web_rocket())
        .await
        .expect("valid rocket instance");

    let dashboard = client.get("/").dispatch().await;
    assert_eq!(dashboard.status(), Status::Ok, "HBS dashboard renders");

    let config = client.get("/ui/config").dispatch().await;
    assert_eq!(config.status(), Status::Ok, "HBS config renders");

    let health = client.get("/ui/health").dispatch().await;
    assert_eq!(health.status(), Status::Ok, "HBS health renders");

    let entities = client.get("/ui/entities").dispatch().await;
    assert_eq!(entities.status(), Status::Ok, "HBS entities renders");
}
