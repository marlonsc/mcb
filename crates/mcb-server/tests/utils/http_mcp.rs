use std::net::TcpListener;
use std::sync::Arc;

use mcb_server::McpServer;
use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::types::{McpRequest, McpResponse};
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use tempfile::TempDir;

use crate::utils::test_fixtures::create_test_mcp_server;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn get_free_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

pub struct McpTestContext {
    pub client: Client,
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

        let rocket = transport.rocket();
        let client = Client::tracked(rocket).await?;

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
) -> TestResult<(Status, McpResponse)> {
    let mut builder = ctx.client.post("/mcp").header(ContentType::JSON);
    for (name, value) in headers {
        builder = builder.header(Header::new((*name).to_owned(), (*value).to_owned()));
    }

    let response = builder
        .body(serde_json::to_string(request)?)
        .dispatch()
        .await;

    let status = response.status();
    let body = response.into_string().await.unwrap_or_default();
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
