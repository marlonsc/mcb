use std::sync::Arc;
use std::time::Duration;

use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};
use mcb_server::transport::axum_http::{AppState, build_router, run_axum_server};
use tokio::time::sleep;

#[tokio::test]
async fn axum_health_check() {
    let probe_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind probe listener");
    let addr = probe_listener
        .local_addr()
        .expect("read probe listener addr");
    drop(probe_listener);

    let state = Arc::new(AppState {
        metrics: AtomicPerformanceMetrics::new_shared(),
        indexing: DefaultIndexingOperations::new_shared(),
        browser: None,
        browse_state: None,
        mcp_server: None,
        admin_state: None,
        auth_config: None,
    });

    let server_state = Arc::clone(&state);
    let server_task = tokio::spawn(async move { run_axum_server(addr, server_state).await });

    let client = reqwest::Client::new();
    let health_url = format!("http://{addr}/health");

    let mut response = None;
    for _ in 0..20 {
        match client.get(&health_url).send().await {
            Ok(ok_response) => {
                response = Some(ok_response);
                break;
            }
            Err(_) => sleep(Duration::from_millis(50)).await,
        }
    }

    let response = response.expect("axum server should accept health request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let payload: serde_json::Value = response.json().await.expect("parse health payload");
    assert_eq!(payload["status"], "healthy");

    let built_router = build_router(state);
    drop(built_router);

    server_task.abort();
}

#[test]
fn app_state_completeness() {
    let state = AppState {
        metrics: AtomicPerformanceMetrics::new_shared(),
        indexing: DefaultIndexingOperations::new_shared(),
        browser: None,
        browse_state: None,
        mcp_server: None,
        admin_state: None,
        auth_config: None,
    };

    assert!(Arc::strong_count(&state.metrics) >= 1);
    assert!(Arc::strong_count(&state.indexing) >= 1);
    assert!(state.browser.is_none());
    assert!(state.browse_state.is_none());
    assert!(state.mcp_server.is_none());
    assert!(state.admin_state.is_none());
    assert!(state.auth_config.is_none());
}
