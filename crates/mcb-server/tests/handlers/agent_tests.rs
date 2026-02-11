use std::sync::Arc;

use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{AgentAction, AgentArgs};
use mcb_server::handlers::AgentHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::test_utils::mock_services::MockAgentSessionService;

#[tokio::test]
async fn test_agent_log_tool_success() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogTool,
        session_id: SessionId::new("test-session"),
        data: json!({
            "tool_name": "search_code",
            "params_summary": "query: test",
            "success": true,
            "duration_ms": 150
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_agent_log_tool_missing_tool_name() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogTool,
        session_id: SessionId::new("test-session"),
        data: json!({
            "params_summary": "query: test",
            "success": true
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing tool_name should return error"
    );
}

#[tokio::test]
async fn test_agent_log_tool_with_error() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogTool,
        session_id: SessionId::new("test-session"),
        data: json!({
            "tool_name": "search_code",
            "success": false,
            "error_message": "Search failed: invalid query",
            "duration_ms": 50
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_agent_log_tool_invalid_data_format() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogTool,
        session_id: SessionId::new("test-session"),
        data: json!("not an object"),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Invalid data format should return error"
    );
}

#[tokio::test]
async fn test_agent_log_delegation_success() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogDelegation,
        session_id: SessionId::new("test-session"),
        data: json!({
            "child_session_id": "child-session-123",
            "agent_type": "search_agent",
            "task_description": "Search for patterns",
            "status": "started"
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_agent_log_delegation_missing_child_session_id() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogDelegation,
        session_id: SessionId::new("test-session"),
        data: json!({
            "agent_type": "search_agent",
            "task_description": "Search for patterns"
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing child_session_id should return error"
    );
}

#[tokio::test]
async fn test_agent_log_delegation_with_result() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogDelegation,
        session_id: SessionId::new("test-session"),
        data: json!({
            "child_session_id": "child-session-456",
            "agent_type": "analysis_agent",
            "task_description": "Analyze code",
            "status": "completed",
            "result_summary": "Found 3 issues"
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_agent_log_delegation_invalid_data_format() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogDelegation,
        session_id: SessionId::new("test-session"),
        data: json!(["not", "an", "object"]),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Invalid data format should return error"
    );
}

#[tokio::test]
async fn test_agent_log_tool_empty_session_id() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogTool,
        session_id: SessionId::new(""),
        data: json!({
            "tool_name": "search_code",
            "success": true
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_err(), "Empty session_id should return McpError");
}

#[tokio::test]
async fn test_agent_log_tool_with_all_optional_fields() {
    let mock_service = MockAgentSessionService::new();
    let handler = AgentHandler::new(Arc::new(mock_service));

    let args = AgentArgs {
        action: AgentAction::LogTool,
        session_id: SessionId::new("test-session"),
        data: json!({
            "tool_name": "index_codebase",
            "params_summary": "path: /home/user/project, collection: main",
            "success": true,
            "error_message": null,
            "duration_ms": 5000
        }),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}
