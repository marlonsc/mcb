//! MCP contract stability: tool names, count, schemas, provenance gates.
//!
//! These tests are the public API contract — if they break, MCP clients break.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::http::StatusCode;
use mcb_server::McpServer;
use mcb_server::tools::{ToolExecutionContext, ToolHandlers, route_tool_call};
use rmcp::model::CallToolRequestParams;
use rstest::rstest;

use mcb_domain::utils::tests::http_mcp::{tools_call_request, tools_list_request};
use mcb_utils::constants::headers::HEADER_WORKSPACE_ROOT;
use mcb_utils::constants::protocol::{
    EXECUTION_FLOW_HYBRID, EXECUTION_FLOW_SERVER_HYBRID, EXECUTION_FLOW_STDIO_ONLY,
    HTTP_HEADER_EXECUTION_FLOW,
};

use crate::utils::http_mcp::{McpTestContext, post_mcp_str};

const EXPECTED_TOOLS: &[&str] = &[
    "analyze_code",
    "analyze_impact",
    "clear_index",
    "compare_branches",
    "entity",
    "get_memories",
    "get_session",
    "index_repo",
    "index_status",
    "inject_context",
    "list_memories",
    "list_repos",
    "list_rules",
    "list_sessions",
    "log_delegation",
    "log_tool_call",
    "memory_timeline",
    "project",
    "search_code",
    "search_memory",
    "start_session",
    "store_memory",
    "summarize_session",
    "validate_code",
];

async fn fetch_tool_list() -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (status, resp) = post_mcp_str(&ctx, &tools_list_request(), &[]).await?;
    assert_eq!(status, StatusCode::OK);
    assert!(resp.error.is_none(), "tools/list must not error");
    let tools = resp
        .result
        .as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or("tools array missing")?
        .clone();
    Ok(tools)
}

fn tool_handlers(server: &Arc<McpServer>) -> ToolHandlers {
    server.tool_handlers()
}

fn provenance_context(delegated: bool) -> ToolExecutionContext {
    ToolExecutionContext {
        session_id: Some("s1".to_owned()),
        parent_session_id: if delegated {
            None
        } else {
            Some("p1".to_owned())
        },
        org_id: None,
        project_id: Some("proj1".to_owned()),
        worktree_id: Some("wt1".to_owned()),
        repo_id: Some("r1".to_owned()),
        repo_path: Some("/tmp/repo".to_owned()),
        operator_id: Some("dev".to_owned()),
        machine_id: Some("laptop".to_owned()),
        agent_program: Some("opencode".to_owned()),
        model_id: Some("gpt-5".to_owned()),
        delegated: Some(delegated),
        timestamp: Some(1),
        execution_flow: Some(EXECUTION_FLOW_STDIO_ONLY.to_owned()),
    }
}

// ─── Tool registry contract ──────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn exactly_24_tools_registered() -> Result<(), Box<dyn std::error::Error>> {
    let tools = fetch_tool_list().await?;
    assert_eq!(tools.len(), 24, "tool count contract changed");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn tool_name_set_matches_contract() -> Result<(), Box<dyn std::error::Error>> {
    let tools = fetch_tool_list().await?;
    let actual: BTreeSet<String> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|v| v.as_str()).map(str::to_owned))
        .collect();
    let expected: BTreeSet<String> = EXPECTED_TOOLS.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(actual, expected, "tool names contract changed");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn every_tool_has_valid_object_schema() -> Result<(), Box<dyn std::error::Error>> {
    let tools = fetch_tool_list().await?;
    for tool in &tools {
        let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let schema = tool.get("inputSchema").ok_or("inputSchema required")?;
        assert!(!schema.is_null(), "'{name}' schema is null");
        assert!(schema.is_object(), "'{name}' schema must be object");
        assert_eq!(
            schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "'{name}' schema.type must be object"
        );
    }
    Ok(())
}

// ─── Provenance gating ───────────────────────────────────────────────

#[rstest]
#[case("index_repo")]
#[case("search_code")]
#[case("store_memory")]
#[tokio::test]
async fn data_plane_tools_reject_empty_provenance(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let req = CallToolRequestParams {
        name: tool_name.to_owned().into(),
        arguments: Some(serde_json::Map::new()),
        task: None,
        meta: None,
    };
    let err = route_tool_call(
        req,
        &tool_handlers(&ctx.server),
        ToolExecutionContext::default(),
    )
    .await
    .expect_err("must reject empty provenance");

    assert_eq!(err.code.0, -32602);
    assert!(err.message.contains("Missing execution provenance"));
    for field in [
        "session_id",
        "repo_id",
        "operator_id",
        "delegated",
        "timestamp",
    ] {
        assert!(err.message.contains(field), "must mention '{field}'");
    }
    Ok(())
}

#[rstest]
#[case("index_repo")]
#[case("search_code")]
#[case("store_memory")]
#[tokio::test]
async fn delegated_agent_without_parent_session_rejected(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let req = CallToolRequestParams {
        name: tool_name.to_owned().into(),
        arguments: Some(serde_json::Map::new()),
        task: None,
        meta: None,
    };
    let err = route_tool_call(req, &tool_handlers(&ctx.server), provenance_context(true))
        .await
        .expect_err("delegated without parent must fail");

    assert_eq!(err.code.0, -32602);
    assert!(err.message.contains("parent_session_id"));
    Ok(())
}

// ─── Operation mode matrix ───────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn validate_blocked_in_server_hybrid_flow() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let headers = [
        (HEADER_WORKSPACE_ROOT, "/tmp"),
        (HTTP_HEADER_EXECUTION_FLOW, EXECUTION_FLOW_SERVER_HYBRID),
    ];
    let (_, resp) = post_mcp_str(&ctx, &tools_call_request("validate_code"), &headers).await?;

    let err = resp.error.ok_or("validate must be blocked")?;
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("Operation mode matrix violation"));
    Ok(())
}

#[rstest]
#[case("search_code")]
#[case("store_memory")]
#[case("list_sessions")]
#[case("log_tool_call")]
#[case("project")]
#[case("list_repos")]
#[case("entity")]
#[tokio::test]
async fn tools_allowed_in_client_hybrid_flow(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let headers = [
        (HEADER_WORKSPACE_ROOT, "/tmp"),
        (HTTP_HEADER_EXECUTION_FLOW, EXECUTION_FLOW_HYBRID),
    ];
    let (_, resp) = post_mcp_str(&ctx, &tools_call_request(tool_name), &headers).await?;

    if let Some(err) = resp.error {
        assert!(
            !err.message.contains("Operation mode matrix violation"),
            "'{tool_name}' should not be blocked in client-hybrid: {}",
            err.message
        );
    }
    Ok(())
}
