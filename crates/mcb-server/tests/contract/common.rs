use axum::http::StatusCode;
use mcb_server::transport::types::{McpRequest, McpResponse};
use serde_json::{Value, json};

use crate::utils::http_mcp::{McpTestContext, post_mcp};

/// Get the workspace root dynamically (works locally and in CI)
pub fn workspace_root() -> String {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| {
            std::path::PathBuf::from(p)
                .join("../..")
                .canonicalize()
                .ok()
        })
        .ok()
        .flatten()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| ".".to_owned())
}

pub fn bridge_headers() -> Vec<(String, String)> {
    let root = workspace_root();
    vec![
        ("X-Workspace-Root".to_owned(), root.clone()),
        ("X-Repo-Path".to_owned(), root),
        ("X-Repo-Id".to_owned(), "contract-test-repo".to_owned()),
        (
            "X-Session-Id".to_owned(),
            "00000000-0000-0000-0000-000000000001".to_owned(),
        ),
        ("X-Project-Id".to_owned(), "project-contract".to_owned()),
        ("X-Worktree-Id".to_owned(), "worktree-contract".to_owned()),
        ("X-Operator-Id".to_owned(), "operator-contract".to_owned()),
        ("X-Machine-Id".to_owned(), "machine-contract".to_owned()),
        ("X-Agent-Program".to_owned(), "contract-suite".to_owned()),
        ("X-Model-Id".to_owned(), "snapshot-model".to_owned()),
        ("X-Delegated".to_owned(), "false".to_owned()),
        ("X-Execution-Flow".to_owned(), "client-hybrid".to_owned()),
    ]
}

pub fn tool_call_request(tool_name: &str, arguments: &Value) -> McpRequest {
    McpRequest {
        method: "tools/call".to_owned(),
        params: Some(json!({
            "name": tool_name,
            "arguments": arguments.clone(),
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
    let workspace_root_str = workspace_root();
    let payload = json!({
        "request": request,
        "status": status.as_u16(),
        "response": response,
    });
    // Normalize paths: replace absolute workspace root with placeholder
    let payload_str = serde_json::to_string(&payload).unwrap_or_default();
    let normalized = payload_str.replace(&workspace_root_str, "<WORKSPACE_ROOT>");
    serde_json::from_str(&normalized).unwrap_or(payload)
}
