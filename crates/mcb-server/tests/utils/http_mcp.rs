use std::net::TcpListener;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mcb_server::McpServer;
use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::types::{McpRequest, McpResponse};
use tempfile::TempDir;
use tower::ServiceExt;

use crate::utils::test_fixtures::create_test_mcp_server;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn get_free_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

pub struct McpTestContext {
    pub client: Router,
    pub server: Arc<McpServer>,
    pub _temp: TempDir,
}

impl McpTestContext {
    pub async fn new() -> TestResult<Self> {
        let port = get_free_port()?;
        let (server_instance, temp) = create_test_mcp_server().await;
        let server = Arc::new(server_instance);

        let http_config = HttpTransportConfig::localhost(port);
        let transport = HttpTransport::new(http_config, Arc::clone(&server));

        let client = transport.router();

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

#[cfg(test)]
mod tests {
    use super::{McpTestContext, get_free_port, post_mcp, tools_call_request, tools_list_request};
    use std::sync::Arc;

    fn server_ref(ctx: &McpTestContext) -> &Arc<mcb_server::McpServer> {
        &ctx.server
    }

    #[test]
    fn http_mcp_symbols_are_linked() {
        let _ = std::mem::size_of::<McpTestContext>();
        let _ = server_ref as for<'a> fn(&'a McpTestContext) -> &'a Arc<mcb_server::McpServer>;
        let _ = get_free_port as fn() -> std::io::Result<u16>;
        let _ = tools_list_request as fn() -> mcb_server::transport::types::McpRequest;
        let _ = tools_call_request as fn(&str) -> mcb_server::transport::types::McpRequest;
        let _ = post_mcp;
        let _ = McpTestContext::new;
    }
}
