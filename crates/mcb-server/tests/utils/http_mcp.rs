use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mcb_server::McpServer;
use mcb_server::transport::http::{HttpTransportState, handle_mcp_request};
use mcb_server::transport::types::{McpRequest, McpResponse};
use tempfile::TempDir;
use tower::ServiceExt;

use crate::utils::test_fixtures::create_test_mcp_server;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub struct McpTestContext {
    pub client: Router,
    pub server: Arc<McpServer>,
    pub _temp: TempDir,
}

impl McpTestContext {
    pub async fn new() -> TestResult<Self> {
        let (server_instance, temp) = create_test_mcp_server().await;
        let server = Arc::new(server_instance);

        let state = Arc::new(HttpTransportState {
            server: Arc::clone(&server),
        });
        let client = Router::new()
            .route("/mcp", axum::routing::post(handle_mcp_request))
            .with_state(state);

        Ok(Self {
            client,
            server,
            _temp: temp,
        })
    }
}

pub async fn post_mcp(
    ctx: &McpTestContext,
    request: &McpRequest,
    headers: &[(&str, &str)],
) -> TestResult<(StatusCode, McpResponse)> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("Content-Type", "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }

    let req = builder.body(Body::from(serde_json::to_vec(request)?))?;
    let response = ctx.client.clone().oneshot(req).await;

    let response = response.map_err(Box::<dyn std::error::Error>::from)?;

    let status = response.status();
    let body = response.into_body().collect().await?.to_bytes();
    let body = String::from_utf8(body.to_vec())?;
    let parsed: McpResponse = serde_json::from_str(&body)?;
    Ok((status, parsed))
}

pub fn tools_list_request() -> McpRequest {
    McpRequest {
        method: "tools/list".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    }
}

pub fn tools_call_request(tool_name: &str) -> McpRequest {
    McpRequest {
        method: "tools/call".to_owned(),
        params: Some(serde_json::json!({
            "name": tool_name,
            "arguments": {}
        })),
        id: Some(serde_json::json!(1)),
    }
}
