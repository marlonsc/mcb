use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{AgentAction, AgentArgs};
use mcb_server::handlers::AgentHandler;
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;
use serde_json::json;

use crate::handlers::utils::create_real_domain_services;
use crate::test_utils::test_fixtures::TEST_SESSION_ID;

async fn create_handler() -> Option<(AgentHandler, tempfile::TempDir)> {
    let (services, temp_dir) = create_real_domain_services().await?;
    Some((AgentHandler::new(services.agent_session_service), temp_dir))
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
        AgentAction::LogTool,
        TEST_SESSION_ID,
        json!({ "success": true })
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
    assert!(result.is_ok());
    let response = result.expect("response");
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
    assert!(result.is_err());
}
