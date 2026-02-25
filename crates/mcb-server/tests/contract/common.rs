use axum::http::StatusCode;
use mcb_server::transport::types::{McpRequest, McpResponse};
use serde_json::{Value, json};

use crate::utils::http_mcp::{McpTestContext, post_mcp};

const WORKTREE: &str = "/home/marlonsc/mcb-v030-seaql-loco-rebuild";

pub fn bridge_headers() -> [(&'static str, &'static str); 12] {
    [
        ("X-Workspace-Root", WORKTREE),
        ("X-Repo-Path", WORKTREE),
        ("X-Repo-Id", "contract-test-repo"),
        ("X-Session-Id", "00000000-0000-0000-0000-000000000001"),
        ("X-Project-Id", "project-contract"),
        ("X-Worktree-Id", "worktree-contract"),
        ("X-Operator-Id", "operator-contract"),
        ("X-Machine-Id", "machine-contract"),
        ("X-Agent-Program", "contract-suite"),
        ("X-Model-Id", "snapshot-model"),
        ("X-Delegated", "false"),
        ("X-Execution-Flow", "client-hybrid"),
    ]
}

#[allow(clippy::needless_pass_by_value)]
pub fn tool_call_request(tool_name: &str, arguments: Value) -> McpRequest {
    McpRequest {
        method: "tools/call".to_owned(),
        params: Some(json!({
            "name": tool_name,
            "arguments": arguments,
        })),
        id: Some(json!(1)),
    }
}

pub async fn call_tool(
    request: &McpRequest,
) -> Result<(StatusCode, McpResponse), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    post_mcp(&ctx, request, &bridge_headers()).await
}

pub fn snapshot_payload(request: &McpRequest, status: StatusCode, response: &McpResponse) -> Value {
    json!({
        "request": request,
        "status": status.as_u16(),
        "response": response,
    })
}
