use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{AgentAction, AgentArgs};
use mcb_server::handlers::AgentHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;
use serde_json::json;

use crate::utils::domain_services::create_real_domain_services;
use crate::utils::test_fixtures::TEST_SESSION_ID;

async fn create_handler() -> Option<(AgentHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_real_domain_services().await?;
    Some((
        AgentHandler::new(state.mcp_server.agent_session_service()),
        temp_dir,
    ))
}

fn build_args(action: AgentAction, session_id: &str, data: serde_json::Value) -> AgentArgs {
    AgentArgs {
        action,
        org_id: None,
        session_id: SessionId::from_name(session_id),
        data,
    }
}

#[rstest]
#[case(
    build_args(
        AgentAction::LogTool,
        TEST_SESSION_ID,
        json!({
            "tool_name": "search_code",
            "params_summary": "query: test",
            "success": true,
            "duration_ms": 150
        })
    )
)]
#[case(
    build_args(
        AgentAction::LogDelegation,
        TEST_SESSION_ID,
        json!({
            "child_session_id": "child-session-123",
            "agent_type": "search_agent",
            "task_description": "Search for patterns",
            "status": "started"
        })
    )
)]
#[tokio::test]
async fn test_agent_actions_return_mcp_response(#[case] args: AgentArgs) {
    let Some((handler, _temp_dir)) = create_handler().await else {
        return;
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("agent handler should succeed for valid agent action");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_agent_log_tool_missing_tool_name_returns_error() {
    let Some((handler, _temp_dir)) = create_handler().await else {
        return;
    };
    let args = build_args(
        AgentAction::LogTool,
        TEST_SESSION_ID,
        json!({ "success": true }),
    );
    let result = handler.handle(Parameters(args)).await;
    let response =
        result.expect("agent handler should return structured validation error response");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_agent_log_tool_empty_session_id() {
    let Some((handler, _temp_dir)) = create_handler().await else {
        return;
    };
    let args = AgentArgs {
        action: AgentAction::LogTool,
        org_id: None,
        session_id: SessionId::from_name(""),
        data: json!({ "tool_name": "search_code", "success": true }),
    };

    let result = handler.handle(Parameters(args)).await;
    let err = result.expect_err("agent handler should fail for empty session_id");
    let err_str = err.to_string();
    assert!(
        err_str.contains("session") || err_str.contains("empty") || err_str.contains("invalid"),
        "error should mention invalid or empty session identifier, got: {err_str}"
    );
}
