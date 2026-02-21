use rstest::rstest;
extern crate mcb_providers;

use std::sync::Arc;

use mcb_server::McpServer;
use mcb_server::tools::router::{ToolExecutionContext, ToolHandlers, route_tool_call};
use rmcp::model::CallToolRequestParams;

use crate::utils::http_mcp::{McpTestContext, post_mcp, tools_call_request};
use crate::utils::test_fixtures::create_test_mcp_server;

fn tool_handlers(server: &Arc<McpServer>) -> ToolHandlers {
    server.tool_handlers()
}

fn empty_call_request(tool_name: &str) -> CallToolRequestParams {
    CallToolRequestParams {
        name: tool_name.to_owned().into(),
        arguments: Some(serde_json::Map::new()),
        task: None,
        meta: None,
    }
}

fn full_provenance_context() -> ToolExecutionContext {
    ToolExecutionContext {
        session_id: Some("ses-test".to_owned()),
        parent_session_id: Some("ses-parent".to_owned()),
        project_id: Some("proj-test".to_owned()),
        worktree_id: Some("wt-test".to_owned()),
        repo_id: Some("repo-test".to_owned()),
        repo_path: Some("/tmp/test-repo".to_owned()),
        operator_id: Some("op-test".to_owned()),
        machine_id: Some("machine-test".to_owned()),
        agent_program: Some("opencode-test".to_owned()),
        model_id: Some("model-test".to_owned()),
        delegated: Some(false),
        timestamp: Some(1700000000),
        execution_flow: Some("stdio-only".to_owned()),
    }
}

#[rstest]
#[case("index")]
#[case("search")]
#[case("validate")]
#[case("memory")]
#[case("session")]
#[case("agent")]
#[case("project")]
#[case("vcs")]
#[case("entity")]
#[tokio::test]
async fn empty_args_returns_invalid_params(#[case] tool_name: &str) {
    let (server, _temp) = create_test_mcp_server().await;
    let handlers = tool_handlers(&Arc::new(server));
    let request = empty_call_request(tool_name);
    let context = full_provenance_context();

    let error = route_tool_call(request, &handlers, context)
        .await
        .expect_err(&format!(
            "{tool_name}: empty args should fail with McpError"
        ));

    assert_eq!(
        error.code.0, -32602,
        "{tool_name}: expected -32602, got {}",
        error.code.0
    );
    assert!(
        error.message.contains("Failed to parse arguments"),
        "{tool_name}: expected parse error, got: {}",
        error.message
    );
}

#[rstest]
#[case("index")]
#[case("search")]
#[case("memory")]
#[tokio::test]
async fn provenance_gated_tools_reject_empty_context(#[case] tool_name: &str) {
    let (server, _temp) = create_test_mcp_server().await;
    let handlers = tool_handlers(&Arc::new(server));
    let request = empty_call_request(tool_name);

    let error = route_tool_call(request, &handlers, ToolExecutionContext::default())
        .await
        .expect_err(&format!("{tool_name}: should reject empty provenance"));

    assert_eq!(error.code.0, -32602);
    assert!(
        error.message.contains("Missing execution provenance"),
        "{tool_name}: expected provenance error, got: {}",
        error.message
    );
}

#[rstest]
#[case("validate")]
#[case("session")]
#[case("agent")]
#[case("project")]
#[case("vcs")]
#[case("entity")]
#[tokio::test]
async fn non_provenance_tools_pass_gate_without_context(#[case] tool_name: &str) {
    let (server, _temp) = create_test_mcp_server().await;
    let handlers = tool_handlers(&Arc::new(server));
    let request = empty_call_request(tool_name);

    let error = route_tool_call(request, &handlers, ToolExecutionContext::default())
        .await
        .expect_err(&format!("{tool_name}: empty args should fail"));

    assert!(
        !error.message.contains("Missing execution provenance"),
        "{tool_name}: should NOT require provenance, got: {}",
        error.message
    );
    assert_eq!(
        error.code.0, -32602,
        "{tool_name}: should still be -32602 from parse failure"
    );
}

#[rstest]
#[case("search")]
#[case("memory")]
#[case("session")]
#[case("agent")]
#[case("project")]
#[case("vcs")]
#[case("entity")]
#[tokio::test]
async fn client_hybrid_allows_server_side_tools(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_call_request(tool_name);
    let headers = [
        ("X-Workspace-Root", "/tmp"),
        ("X-Execution-Flow", "client-hybrid"),
    ];
    let (status, response) = post_mcp(&ctx, &request, &headers).await?;

    assert_eq!(status, rocket::http::Status::Ok);
    let error_opt = response.error;
    // Tools should now be allowed through the matrix check in client-hybrid mode.
    // They may fail for other reasons (e.g., missing arguments), but NOT due to mode violation.
    if let Some(error) = error_opt {
        assert!(
            !error.message.contains("Operation mode matrix violation"),
            "{tool_name}: should NOT have mode violation in client-hybrid, got: {}",
            error.message
        );
    }
    Ok(())
}

#[tokio::test]
async fn server_hybrid_blocks_validate() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_call_request("validate");
    let headers = [
        ("X-Workspace-Root", "/tmp"),
        ("X-Execution-Flow", "server-hybrid"),
    ];
    let (status, response) = post_mcp(&ctx, &request, &headers).await?;

    assert_eq!(status, rocket::http::Status::Ok);
    let error_opt = response.error;
    assert!(
        error_opt.is_some(),
        "validate should be blocked in server-hybrid"
    );
    let error = match error_opt {
        Some(error) => error,
        None => return Ok(()),
    };
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Operation mode matrix violation"));
    Ok(())
}
