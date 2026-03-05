use axum::http::StatusCode;
use mcb_domain::protocol::{JSONRPC_VERSION, McpRequest, McpResponse};
use mcb_utils::constants::headers::*;
use mcb_utils::constants::protocol::{EXECUTION_FLOW_HYBRID, HTTP_HEADER_EXECUTION_FLOW};
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
        (HEADER_WORKSPACE_ROOT.to_owned(), root.clone()),
        (HEADER_REPO_PATH.to_owned(), root),
        (HEADER_REPO_ID.to_owned(), "contract-test-repo".to_owned()),
        (
            HEADER_SESSION_ID.to_owned(),
            "00000000-0000-0000-0000-000000000001".to_owned(),
        ),
        (HEADER_PROJECT_ID.to_owned(), "project-contract".to_owned()),
        (
            HEADER_WORKTREE_ID.to_owned(),
            "worktree-contract".to_owned(),
        ),
        (
            HEADER_OPERATOR_ID.to_owned(),
            "operator-contract".to_owned(),
        ),
        (HEADER_MACHINE_ID.to_owned(), "machine-contract".to_owned()),
        (HEADER_AGENT_PROGRAM.to_owned(), "contract-suite".to_owned()),
        (HEADER_MODEL_ID.to_owned(), "snapshot-model".to_owned()),
        (HEADER_DELEGATED.to_owned(), "false".to_owned()),
        (
            HTTP_HEADER_EXECUTION_FLOW.to_owned(),
            EXECUTION_FLOW_HYBRID.to_owned(),
        ),
    ]
}

pub fn tool_call_request(tool_name: &str, arguments: &Value) -> McpRequest {
    McpRequest {
        jsonrpc: JSONRPC_VERSION.to_owned(),
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
