use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{SessionAction, SessionArgs};
use mcb_server::handlers::SessionHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::handlers::utils::create_real_domain_services;
use crate::test_utils::test_fixtures::TEST_PROJECT_ID;
use crate::test_utils::text::extract_text;

async fn create_handler() -> Option<(SessionHandler, tempfile::TempDir)> {
    let (services, temp_dir) = create_real_domain_services().await?;
    Some((
        SessionHandler::new(services.agent_session_service, services.memory_service),
        temp_dir,
    ))
}

macro_rules! session_test {
    ($test_name:ident, $action:expr, session_id: $session_id:expr, expect_ok) => {
        #[tokio::test]
        async fn $test_name() {
            let Some((handler, _services_temp_dir)) = create_handler().await else {
                return;
            };

            let args = SessionArgs {
                action: $action,
                org_id: None,
                session_id: Some($session_id),
                project_id: None,
                data: None,
                worktree_id: None,
                parent_session_id: None,
                agent_type: None,
                status: None,
                limit: None,
            };

            let result = handler.handle(Parameters(args)).await;
            assert!(result.is_ok());
            let _response = result.expect("Expected response");
        }
    };

    ($test_name:ident, $action:expr, data: $data:expr, $(agent_type: $agent_type:expr,)? expect_ok) => {
        #[tokio::test]
        async fn $test_name() {
            let Some((handler, _services_temp_dir)) = create_handler().await else {
                return;
            };

            let args = SessionArgs {
                action: $action,
                org_id: None,
                session_id: None,
                project_id: None,
                data: Some($data),
                worktree_id: None,
                parent_session_id: None,
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

    ($test_name:ident, $action:expr, data: $data:expr, $(agent_type: $agent_type:expr,)? expect_error) => {
        #[tokio::test]
        async fn $test_name() {
            let Some((handler, _services_temp_dir)) = create_handler().await else {
                return;
            };

            let args = SessionArgs {
                action: $action,
                org_id: None,
                session_id: None,
                project_id: None,
                data: $data,
                worktree_id: None,
                parent_session_id: None,
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
        "project_id": TEST_PROJECT_ID
    }),
    agent_type: Some("explore".to_owned()),
    expect_ok
);

session_test!(
    test_session_create_without_session_summary_id_success,
    SessionAction::Create,
    data: json!({
        "model": "claude-3-sonnet",
        "project_id": TEST_PROJECT_ID
    }),
    agent_type: Some("explore".to_owned()),
    expect_ok
);

session_test!(
    test_session_create_missing_data,
    SessionAction::Create,
    data: None,
    agent_type: Some("explore".to_owned()),
    expect_error
);

session_test!(
    test_session_create_invalid_data,
    SessionAction::Create,
    data: Some(json!("not an object")),
    agent_type: Some("explore".to_owned()),
    expect_error
);

session_test!(
    test_session_get_success,
    SessionAction::Get,
    session_id: SessionId::from_name("test-session-id"),
    expect_ok
);

session_test!(
    test_session_get_nonexistent_session,
    SessionAction::Get,
    session_id: SessionId::from_name("nonexistent-session"),
    expect_ok
);

session_test!(
    test_session_summarize_success,
    SessionAction::Summarize,
    session_id: SessionId::from_name("test-session-id"),
    expect_ok
);

session_test!(
    test_session_summarize_nonexistent_session,
    SessionAction::Summarize,
    session_id: SessionId::from_name("nonexistent-session"),
    expect_ok
);

#[tokio::test]
async fn test_session_update_conflicting_project_id_rejected() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };

    let create_args = SessionArgs {
        action: SessionAction::Create,
        org_id: None,
        session_id: None,
        project_id: Some("project-a".to_owned()),
        data: Some(json!({
            "session_summary_id": "summary-update-conflict",
            "model": "claude-3-sonnet",
            "project_id": "project-a"
        })),
        worktree_id: None,
        parent_session_id: None,
        agent_type: Some("explore".to_owned()),
        status: None,
        limit: None,
    };

    let create_result = handler
        .handle(Parameters(create_args))
        .await
        .expect("create session must succeed");
    assert!(!create_result.is_error.unwrap_or(false));

    let created_text = extract_text(&create_result.content);
    let created_json: serde_json::Value =
        serde_json::from_str(&created_text).expect("create response json");
    let session_id = created_json
        .get("session_id")
        .and_then(|v| v.as_str())
        .expect("session_id in create response")
        .to_owned();

    let update_args = SessionArgs {
        action: SessionAction::Update,
        org_id: None,
        session_id: Some(SessionId::from_string(&session_id)),
        project_id: Some("project-b".to_owned()),
        data: Some(json!({
            "status": "completed"
        })),
        worktree_id: None,
        parent_session_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };

    let update_result = handler.handle(Parameters(update_args)).await;
    let err = update_result.expect_err("conflicting project_id must return invalid_params");
    assert!(err.message.contains("conflicting project_id"));
}
