use rstest::rstest;
extern crate mcb_providers;

use std::collections::BTreeSet;
use std::sync::Arc;

use mcb_server::McpServer;
use mcb_server::tools::router::{ToolExecutionContext, ToolHandlers, route_tool_call};
use rmcp::model::CallToolRequestParams;
use rocket::http::Status;

use crate::utils::http_mcp::{McpTestContext, post_mcp, tools_call_request, tools_list_request};

fn tool_handlers(server: &Arc<McpServer>) -> ToolHandlers {
    server.tool_handlers()
}

fn direct_tool_call_request(tool_name: &str) -> CallToolRequestParams {
    CallToolRequestParams {
        name: tool_name.to_owned().into(),
        arguments: Some(serde_json::Map::new()),
        task: None,
        meta: None,
    }
}

#[rstest]
#[tokio::test]
async fn test_tool_name_set_stability() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_list_request();
    let (status, response) = post_mcp(&ctx, &request, &[]).await?;

    assert_eq!(status, Status::Ok);
    assert!(response.error.is_none(), "tools/list should not error");

    let result_opt = response.result;
    assert!(result_opt.is_some(), "tools/list result");
    let result = match result_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    let tools_opt = result.get("tools").and_then(serde_json::Value::as_array);
    assert!(tools_opt.is_some(), "tools array");
    let tools = match tools_opt {
        Some(values) => values,
        None => return Ok(()),
    };

    let actual: BTreeSet<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name").and_then(|v| v.as_str()).map(str::to_owned))
        .collect();

    let expected: BTreeSet<String> = [
        "agent", "entity", "index", "memory", "project", "search", "session", "validate", "vcs",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();

    assert_eq!(actual, expected, "tool names contract changed");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_tool_count_stability() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_list_request();
    let (status, response) = post_mcp(&ctx, &request, &[]).await?;

    assert_eq!(status, Status::Ok);
    assert!(response.error.is_none(), "tools/list should not error");

    let result_opt = response.result;
    assert!(result_opt.is_some(), "tools/list result");
    let result = match result_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    let tools_opt = result.get("tools").and_then(serde_json::Value::as_array);
    assert!(tools_opt.is_some(), "tools array");
    let tools = match tools_opt {
        Some(values) => values,
        None => return Ok(()),
    };
    assert_eq!(tools.len(), 9, "tool count contract changed");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_each_tool_has_non_null_object_input_schema_with_properties()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_list_request();
    let (status, response) = post_mcp(&ctx, &request, &[]).await?;

    assert_eq!(status, Status::Ok);
    assert!(response.error.is_none(), "tools/list should not error");

    let result_opt = response.result;
    assert!(result_opt.is_some(), "tools/list result");
    let result = match result_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    let tools_opt = result.get("tools").and_then(serde_json::Value::as_array);
    assert!(tools_opt.is_some(), "tools array");
    let tools = match tools_opt {
        Some(values) => values,
        None => return Ok(()),
    };

    for tool in tools {
        let name_opt = tool.get("name").and_then(serde_json::Value::as_str);
        assert!(name_opt.is_some(), "tool name string");
        let name = match name_opt {
            Some(value) => value,
            None => continue,
        };
        let schema_opt = tool.get("inputSchema");
        assert!(schema_opt.is_some(), "inputSchema must exist");
        let schema = match schema_opt {
            Some(value) => value,
            None => continue,
        };

        assert!(!schema.is_null(), "{name} inputSchema must not be null");
        assert!(schema.is_object(), "{name} inputSchema must be object");
        assert_eq!(
            schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "{name} inputSchema.type must be object"
        );
        assert!(
            schema
                .get("properties")
                .is_some_and(serde_json::Value::is_object),
            "{name} inputSchema.properties must be object"
        );
    }
    Ok(())
}

#[rstest]
#[case("index")]
#[case("search")]
#[case("memory")]
#[tokio::test]
async fn test_provenance_gating_requires_full_provenance_fields(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let handlers = tool_handlers(&ctx.server);
    let request = direct_tool_call_request(tool_name);
    let error_result = route_tool_call(request, &handlers, ToolExecutionContext::default()).await;
    assert!(
        error_result.is_err(),
        "provenance-gated tools should fail without full provenance"
    );
    let error = match error_result {
        Ok(_) => return Ok(()),
        Err(error) => error,
    };

    assert_eq!(error.code.0, -32602);
    assert!(error.message.contains("Missing execution provenance"));
    for field in [
        "session_id",
        "project_id",
        "repo_id",
        "repo_path",
        "worktree_id",
        "operator_id",
        "machine_id",
        "agent_program",
        "model_id",
        "delegated",
        "timestamp",
    ] {
        assert!(
            error.message.contains(field),
            "error should mention missing {field}: {}",
            error.message
        );
    }
    Ok(())
}

#[rstest]
#[case("index")]
#[case("search")]
#[case("memory")]
#[tokio::test]
async fn test_delegation_requires_parent_session_id_when_delegated_true(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let handlers = tool_handlers(&ctx.server);
    let request = direct_tool_call_request(tool_name);
    let execution_context = ToolExecutionContext {
        session_id: Some("session-1".to_owned()),
        parent_session_id: None,
        project_id: Some("project-1".to_owned()),
        worktree_id: Some("worktree-1".to_owned()),
        repo_id: Some("repo-1".to_owned()),
        repo_path: Some("/tmp/repo".to_owned()),
        operator_id: Some("operator-1".to_owned()),
        machine_id: Some("machine-1".to_owned()),
        agent_program: Some("opencode".to_owned()),
        model_id: Some("gpt-5.3-codex".to_owned()),
        delegated: Some(true),
        timestamp: Some(1),
        execution_flow: Some("stdio-only".to_owned()),
    };
    let error_result = route_tool_call(request, &handlers, execution_context).await;
    assert!(
        error_result.is_err(),
        "delegated=true without parent_session_id should fail"
    );
    let error = match error_result {
        Ok(_) => return Ok(()),
        Err(error) => error,
    };

    assert_eq!(error.code.0, -32602);
    assert!(
        error.message.contains("parent_session_id"),
        "error should mention missing parent_session_id: {}",
        error.message
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_operation_mode_matrix_blocks_validate_in_server_hybrid()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_call_request("validate");
    let headers = [
        ("X-Workspace-Root", "/tmp"),
        ("X-Execution-Flow", "server-hybrid"),
    ];
    let (status, response) = post_mcp(&ctx, &request, &headers).await?;

    assert_eq!(status, Status::Ok);
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
    assert!(error.message.contains("validate"));
    assert!(error.message.contains("server-hybrid"));
    Ok(())
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
async fn test_operation_mode_matrix_blocks_tools_in_client_hybrid(
    #[case] tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = tools_call_request(tool_name);
    let headers = [
        ("X-Workspace-Root", "/tmp"),
        ("X-Execution-Flow", "client-hybrid"),
    ];
    let (status, response) = post_mcp(&ctx, &request, &headers).await?;

    assert_eq!(status, Status::Ok);
    let error_opt = response.error;
    assert!(
        error_opt.is_some(),
        "tool should be blocked in client-hybrid"
    );
    let error = match error_opt {
        Some(error) => error,
        None => return Ok(()),
    };
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Operation mode matrix violation"));
    assert!(error.message.contains(tool_name));
    assert!(error.message.contains("client-hybrid"));
    Ok(())
}
