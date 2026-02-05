use mcb_server::args::{SessionAction, SessionArgs};
use mcb_server::handlers::SessionHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;
use std::sync::Arc;

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
            "agent_type": "test_agent",
            "project_id": "test-project"
        })),
        project_id: Some("test-project".to_string()),
        agent_type: Some("test_agent".to_string()),
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
        agent_type: Some("test_agent".to_string()),
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
        agent_type: Some("test_agent".to_string()),
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
        session_id: Some("test-session-id".to_string()),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_get_missing_session_id() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Get,
        session_id: None,
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing session_id should return error"
    );
}

#[tokio::test]
async fn test_session_get_nonexistent_session() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Get,
        session_id: Some("nonexistent-session".to_string()),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
}

#[tokio::test]
async fn test_session_update_success() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Update,
        session_id: Some("test-session-id".to_string()),
        data: Some(json!({
            "status": "in_progress",
            "metadata": {}
        })),
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_update_missing_session_id() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Update,
        session_id: None,
        data: Some(json!({"status": "in_progress"})),
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing session_id should return error"
    );
}

#[tokio::test]
async fn test_session_update_missing_data() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Update,
        session_id: Some("test-session-id".to_string()),
        data: None,
        project_id: None,
        agent_type: None,
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
async fn test_session_list_success() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::List,
        session_id: None,
        data: None,
        project_id: Some("test-project".to_string()),
        agent_type: Some("test_agent".to_string()),
        status: Some("active".to_string()),
        limit: Some(10),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_list_no_filters() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::List,
        session_id: None,
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_list_with_limit() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::List,
        session_id: None,
        data: None,
        project_id: Some("test-project".to_string()),
        agent_type: None,
        status: None,
        limit: Some(5),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_summarize_success() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Summarize,
        session_id: Some("test-session-id".to_string()),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_session_summarize_missing_session_id() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Summarize,
        session_id: None,
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing session_id should return error"
    );
}

#[tokio::test]
async fn test_session_summarize_nonexistent_session() {
    let agent_service = MockAgentSessionService::new();
    let memory_service = MockMemoryService::new();
    let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

    let args = SessionArgs {
        action: SessionAction::Summarize,
        session_id: Some("nonexistent-session".to_string()),
        data: None,
        project_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
}
