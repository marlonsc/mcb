extern crate mcb_providers;

use std::collections::BTreeSet;
use std::net::TcpListener;
use std::sync::Arc;

use mcb_server::McpServer;
use mcb_server::tools::router::{ToolExecutionContext, ToolHandlers, route_tool_call};
use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::types::{McpRequest, McpResponse};
use rmcp::model::CallToolRequestParams;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use rstest::*;
use tempfile::TempDir;

use crate::test_utils::test_fixtures::create_test_mcp_server;

fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();
    drop(listener);
    port
}

struct TestContext {
    client: Client,
    server: Arc<McpServer>,
    _temp: TempDir,
}

#[fixture]
async fn ctx() -> TestContext {
    let port = get_free_port();
    let (server_instance, temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server.clone());

    let rocket = transport.rocket();
    let client = Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    TestContext {
        client,
        server,
        _temp: temp,
    }
}

fn tool_handlers(server: &Arc<McpServer>) -> ToolHandlers {
    ToolHandlers {
        index: server.index_handler(),
        search: server.search_handler(),
        validate: server.validate_handler(),
        memory: server.memory_handler(),
        session: server.session_handler(),
        agent: server.agent_handler(),
        project: server.project_handler(),
        vcs: server.vcs_handler(),
        vcs_entity: server.vcs_entity_handler(),
        plan_entity: server.plan_entity_handler(),
        issue_entity: server.issue_entity_handler(),
        org_entity: server.org_entity_handler(),
        entity: server.entity_handler(),
        hook_processor: server.hook_processor(),
    }
}

fn direct_tool_call_request(tool_name: &str) -> CallToolRequestParams {
    CallToolRequestParams {
        name: tool_name.to_string().into(),
        arguments: Some(serde_json::Map::new()),
        task: None,
        meta: None,
    }
}

async fn post_mcp(
    ctx: &TestContext,
    request: &McpRequest,
    headers: &[(&str, &str)],
) -> (Status, McpResponse) {
    let mut builder = ctx.client.post("/mcp").header(ContentType::JSON);
    for (name, value) in headers {
        builder = builder.header(Header::new((*name).to_string(), (*value).to_string()));
    }

    let response = builder
        .body(serde_json::to_string(request).expect("serialize request"))
        .dispatch()
        .await;

    let status = response.status();
    let body = response.into_string().await.expect("Response body");
    let parsed: McpResponse = serde_json::from_str(&body).expect("Parse response");
    (status, parsed)
}

fn tools_list_request() -> McpRequest {
    McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    }
}

fn tools_call_request(tool_name: &str) -> McpRequest {
    McpRequest {
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": tool_name,
            "arguments": {}
        })),
        id: Some(serde_json::json!(1)),
    }
}

#[rstest]
#[tokio::test]
async fn test_tool_name_set_stability(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let request = tools_list_request();
    let (status, response) = post_mcp(&ctx, &request, &[]).await;

    assert_eq!(status, Status::Ok);
    assert!(response.error.is_none(), "tools/list should not error");

    let result = response.result.expect("tools/list result");
    let tools = result
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools array");

    let actual: BTreeSet<&str> = tools
        .iter()
        .map(|tool| {
            tool.get("name")
                .and_then(|v| v.as_str())
                .expect("tool name must be string")
        })
        .collect();

    let expected: BTreeSet<&str> = [
        "agent", "entity", "index", "memory", "project", "search", "session", "validate", "vcs",
    ]
    .into();

    assert_eq!(actual, expected, "tool names contract changed");
}

#[rstest]
#[tokio::test]
async fn test_tool_count_stability(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let request = tools_list_request();
    let (status, response) = post_mcp(&ctx, &request, &[]).await;

    assert_eq!(status, Status::Ok);
    assert!(response.error.is_none(), "tools/list should not error");

    let result = response.result.expect("tools/list result");
    let tools = result
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools array");
    assert_eq!(tools.len(), 9, "tool count contract changed");
}

#[rstest]
#[tokio::test]
async fn test_each_tool_has_non_null_object_input_schema_with_properties(
    #[future] ctx: TestContext,
) {
    let ctx = ctx.await;
    let request = tools_list_request();
    let (status, response) = post_mcp(&ctx, &request, &[]).await;

    assert_eq!(status, Status::Ok);
    assert!(response.error.is_none(), "tools/list should not error");

    let result = response.result.expect("tools/list result");
    let tools = result
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools array");

    for tool in tools {
        let name = tool
            .get("name")
            .and_then(|v| v.as_str())
            .expect("tool name string");
        let schema = tool.get("inputSchema").expect("inputSchema must exist");

        assert!(!schema.is_null(), "{name} inputSchema must not be null");
        assert!(schema.is_object(), "{name} inputSchema must be object");
        assert_eq!(
            schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "{name} inputSchema.type must be object"
        );
        assert!(
            schema.get("properties").is_some_and(|v| v.is_object()),
            "{name} inputSchema.properties must be object"
        );
    }
}

#[rstest]
#[case("index")]
#[case("search")]
#[case("memory")]
#[tokio::test]
async fn test_provenance_gating_requires_full_provenance_fields(
    #[future] ctx: TestContext,
    #[case] tool_name: &str,
) {
    let ctx = ctx.await;
    let handlers = tool_handlers(&ctx.server);
    let request = direct_tool_call_request(tool_name);
    let error = route_tool_call(request, &handlers, ToolExecutionContext::default())
        .await
        .expect_err("provenance-gated tools should fail without full provenance");

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
}

#[rstest]
#[case("index")]
#[case("search")]
#[case("memory")]
#[tokio::test]
async fn test_delegation_requires_parent_session_id_when_delegated_true(
    #[future] ctx: TestContext,
    #[case] tool_name: &str,
) {
    let ctx = ctx.await;
    let handlers = tool_handlers(&ctx.server);
    let request = direct_tool_call_request(tool_name);
    let execution_context = ToolExecutionContext {
        session_id: Some("session-1".to_string()),
        parent_session_id: None,
        project_id: Some("project-1".to_string()),
        worktree_id: Some("worktree-1".to_string()),
        repo_id: Some("repo-1".to_string()),
        repo_path: Some("/tmp/repo".to_string()),
        operator_id: Some("operator-1".to_string()),
        machine_id: Some("machine-1".to_string()),
        agent_program: Some("opencode".to_string()),
        model_id: Some("gpt-5.3-codex".to_string()),
        delegated: Some(true),
        timestamp: Some(1),
        execution_flow: Some("stdio-only".to_string()),
    };
    let error = route_tool_call(request, &handlers, execution_context)
        .await
        .expect_err("delegated=true without parent_session_id should fail");

    assert_eq!(error.code.0, -32602);
    assert!(
        error.message.contains("parent_session_id"),
        "error should mention missing parent_session_id: {}",
        error.message
    );
}

#[rstest]
#[tokio::test]
async fn test_operation_mode_matrix_blocks_validate_in_server_hybrid(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let request = tools_call_request("validate");
    let headers = [
        ("X-Workspace-Root", "/tmp"),
        ("X-Execution-Flow", "server-hybrid"),
    ];
    let (status, response) = post_mcp(&ctx, &request, &headers).await;

    assert_eq!(status, Status::Ok);
    let error = response
        .error
        .expect("validate should be blocked in server-hybrid");
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Operation mode matrix violation"));
    assert!(error.message.contains("validate"));
    assert!(error.message.contains("server-hybrid"));
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
    #[future] ctx: TestContext,
    #[case] tool_name: &str,
) {
    let ctx = ctx.await;
    let request = tools_call_request(tool_name);
    let headers = [
        ("X-Workspace-Root", "/tmp"),
        ("X-Execution-Flow", "client-hybrid"),
    ];
    let (status, response) = post_mcp(&ctx, &request, &headers).await;

    assert_eq!(status, Status::Ok);
    let error = response
        .error
        .expect("tool should be blocked in client-hybrid");
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Operation mode matrix violation"));
    assert!(error.message.contains(tool_name));
    assert!(error.message.contains("client-hybrid"));
}
