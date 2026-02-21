//! Axum test harness — in-process request dispatch via `tower::ServiceExt::oneshot`.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::transport::axum_http::{AppState, build_router};
use tower::ServiceExt;

#[must_use]
pub fn test_app() -> Router {
    let state = Arc::new(AppState {
        metrics: AtomicPerformanceMetrics::new_shared(),
        indexing: DefaultIndexingOperations::new_shared(),
        browser: None,
        browse_state: None,
        mcp_server: None,
        admin_state: None,
        auth_config: None,
    });
    build_router(state)
}

#[must_use]
pub fn test_app_with_state(state: Arc<AppState>) -> Router {
    build_router(state)
}

pub struct TestResponse {
    pub status: StatusCode,
    body: Vec<u8>,
}

impl TestResponse {
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8(self.body.clone()).expect("response body should be valid UTF-8")
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> T {
        serde_json::from_slice(&self.body).expect("response body should be valid JSON")
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }
}

pub async fn test_get(app: &Router, path: &str) -> TestResponse {
    let request = Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("valid GET request");

    dispatch(app.clone(), request).await
}

pub async fn test_post(app: &Router, path: &str, body: &str) -> TestResponse {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_owned()))
        .expect("valid POST request");

    dispatch(app.clone(), request).await
}

pub async fn test_get_auth(app: &Router, path: &str, api_key: &str) -> TestResponse {
    let request = Request::builder()
        .method("GET")
        .uri(path)
        .header("X-Admin-Key", api_key)
        .body(Body::empty())
        .expect("valid authenticated GET request");

    dispatch(app.clone(), request).await
}

pub async fn test_post_auth(app: &Router, path: &str, body: &str, api_key: &str) -> TestResponse {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("Content-Type", "application/json")
        .header("X-Admin-Key", api_key)
        .body(Body::from(body.to_owned()))
        .expect("valid authenticated POST request");

    dispatch(app.clone(), request).await
}

async fn dispatch(app: Router, request: Request<Body>) -> TestResponse {
    let response = app
        .oneshot(request)
        .await
        .expect("router should handle request");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("should collect response body")
        .to_bytes()
        .to_vec();

    TestResponse { status, body }
}
