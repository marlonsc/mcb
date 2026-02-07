use std::sync::Arc;

use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{SessionAction, SessionArgs};
use mcb_server::handlers::SessionHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::test_utils::mock_services::{MockAgentSessionService, MockMemoryService};

#[tokio::test]
async fn test_session_create_success() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Create,
        session_id: None,
        data: Some(json!({
            "session_summary_id": "summary-123",
            "model": "claude-3-sonnet",
            "project_id": "test-project"
        })),
        project_id: Some("test-project".to_string()),
        agent_type: Some("explore".to_string()),
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_create_missing_data() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Create,
        session_id: None,
        data: None,
        project_id: Some("test-project".to_string()),
        agent_type: Some("explore".to_string()),
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing data should return error"
    );
}

#[tokio::test]
async fn test_session_create_invalid_data() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Create,
        session_id: None,
        data: Some(json!("not an object")),
        project_id: Some("test-project".to_string()),
        agent_type: Some("explore".to_string()),
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Invalid data should return error"
    );
}

#[tokio::test]
async fn test_session_get_success() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Get,
        session_id: Some(SessionId::new("test-session-id")),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[tokio::test]
async fn test_session_get_nonexistent_session() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Get,
        session_id: Some(SessionId::new("nonexistent-session")),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[tokio::test]
async fn test_session_summarize_success() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Summarize,
        session_id: Some(SessionId::new("test-session-id")),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}

#[tokio::test]
async fn test_session_summarize_nonexistent_session() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Summarize,
        session_id: Some(SessionId::new("nonexistent-session")),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected response");
}
