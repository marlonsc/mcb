use std::sync::Arc;

use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{SessionAction, SessionArgs};
use mcb_server::handlers::SessionHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::test_utils::mock_services::{MockAgentSessionService, MockMemoryService};

macro_rules! session_test {
    ($test_name:ident, $action:expr, session_id: $session_id:expr, expect_ok) => {
        #[tokio::test]
        async fn $test_name() {
            let agent_service = MockAgentSessionService::new();
            let memory_service = MockMemoryService::new();
            let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

            let args = SessionArgs {
                org_id: None,
                action: $action,
                session_id: Some($session_id),
                org_id: None,
                data: None,
                project_id: None,
                worktree_id: None,
                agent_type: None,
                status: None,
                limit: None,
            };

            let result = handler.handle(Parameters(args)).await;
            assert!(result.is_ok());
            let _response = result.expect("Expected response");
        }
    };

    ($test_name:ident, $action:expr, data: $data:expr, $(project_id: $project_id:expr,)? $(agent_type: $agent_type:expr,)? expect_ok) => {
        #[tokio::test]
        async fn $test_name() {
            let agent_service = MockAgentSessionService::new();
            let memory_service = MockMemoryService::new();
            let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

            let args = SessionArgs {
                org_id: None,
                action: $action,
                session_id: None,
                org_id: None,
                data: Some($data),
                project_id: None $(.or($project_id))?,
                worktree_id: None,
                agent_type: None $(.or($agent_type))?,
                status: None,
                limit: None,
            };

            let result = handler.handle(Parameters(args)).await;
            assert!(result.is_ok());
            let response = result.expect("Expected successful response");
            assert!(!response.is_error.unwrap_or(false));
        }
    };

    ($test_name:ident, $action:expr, data: $data:expr, $(project_id: $project_id:expr,)? $(agent_type: $agent_type:expr,)? expect_error) => {
        #[tokio::test]
        async fn $test_name() {
            let agent_service = MockAgentSessionService::new();
            let memory_service = MockMemoryService::new();
            let handler = SessionHandler::new(Arc::new(agent_service), Arc::new(memory_service));

            let args = SessionArgs {
                org_id: None,
                action: $action,
                session_id: None,
                org_id: None,
                data: $data,
                project_id: None $(.or($project_id))?,
                worktree_id: None,
                agent_type: None $(.or($agent_type))?,
                status: None,
                limit: None,
            };

            let result = handler.handle(Parameters(args)).await;
            assert!(result.is_ok());
            let response = result.expect("Expected response");
            assert!(response.is_error.unwrap_or(false), "Should return error");
        }
    };
}

session_test!(
    test_session_create_success,
    SessionAction::Create,
    data: json!({
        "session_summary_id": "summary-123",
        "model": "claude-3-sonnet",
        "project_id": "test-project"
    }),
    project_id: Some("test-project".to_string()),
    agent_type: Some("explore".to_string()),
    expect_ok
);

session_test!(
    test_session_create_missing_data,
    SessionAction::Create,
    data: None,
    project_id: Some("test-project".to_string()),
    agent_type: Some("explore".to_string()),
    expect_error
);

session_test!(
    test_session_create_invalid_data,
    SessionAction::Create,
    data: Some(json!("not an object")),
    project_id: Some("test-project".to_string()),
    agent_type: Some("explore".to_string()),
    expect_error
);

session_test!(
    test_session_get_success,
    SessionAction::Get,
    session_id: SessionId::new("test-session-id"),
    expect_ok
);

session_test!(
    test_session_get_nonexistent_session,
    SessionAction::Get,
    session_id: SessionId::new("nonexistent-session"),
    expect_ok
);

session_test!(
    test_session_summarize_success,
    SessionAction::Summarize,
    session_id: SessionId::new("test-session-id"),
    expect_ok
);

session_test!(
    test_session_summarize_nonexistent_session,
    SessionAction::Summarize,
    session_id: SessionId::new("nonexistent-session"),
    expect_ok
);
