//! Error shape tests for MCP handler error responses.

use rstest::{fixture, rstest};

use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource, SessionAction, SessionArgs};
use mcb_server::handlers::{MemoryHandler, SessionHandler};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::{Value, json};

use crate::utils::test_fixtures::{create_base_memory_args, create_test_mcb_state};
use mcb_domain::utils::tests::mcp_assertions::assert_error_shape;

async fn memory_handler() -> Option<(MemoryHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_test_mcb_state().await?;
    Some((
        MemoryHandler::new(state.mcp_server.memory_service()),
        temp_dir,
    ))
}

async fn session_handler() -> Option<(SessionHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_test_mcb_state().await?;
    Some((
        SessionHandler::new(
            state.mcp_server.agent_session_service(),
            state.mcp_server.memory_service(),
        ),
        temp_dir,
    ))
}

#[fixture]
fn observation_store_args() -> MemoryArgs {
    create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        None,
        None,
        Some("session-1".to_owned()),
    )
}

#[rstest]
#[tokio::test]
async fn memory_store_missing_data_returns_expected_error(
    #[from(observation_store_args)] args: MemoryArgs,
) {
    let Some((handler, _temp_dir)) = memory_handler().await else {
        return;
    };
    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");

    assert_error_shape(&response, "Missing data payload for observation store");
}

#[rstest]
#[tokio::test]
async fn memory_store_missing_content_returns_expected_error(
    #[from(observation_store_args)] mut args: MemoryArgs,
) {
    args.data = Some(json!({
        "observation_type": "code",
        "project_id": "project-1"
    }));

    let Some((handler, _temp_dir)) = memory_handler().await else {
        return;
    };
    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");

    assert_error_shape(&response, "Missing required field: content");
}

#[rstest]
#[tokio::test]
async fn session_create_missing_data_returns_expected_error() {
    let Some((handler, _temp_dir)) = session_handler().await else {
        return;
    };
    let args = SessionArgs {
        action: SessionAction::Create,
        org_id: None,
        session_id: None,
        data: None,
        project_id: None,
        worktree_id: None,
        parent_session_id: None,
        agent_type: Some("explore".to_owned()),
        status: None,
        limit: None,
    };

    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");
    assert_error_shape(&response, "Missing data payload for create");
}

#[rstest]
#[case(json!({"action": "bad_action", "resource": "observation"}), "action")]
#[case(json!({"action": "store", "resource": "bad_resource"}), "resource")]
fn invalid_enum_payloads_fail_deserialization(#[case] payload: Value, #[case] expected: &str) {
    let err = serde_json::from_value::<MemoryArgs>(payload).expect_err("invalid enum should fail");
    let message = err.to_string();

    assert!(
        message.contains(expected),
        "expected '{expected}' in deserialization error: {message}"
    );
    assert!(
        message.contains("unknown variant") || message.contains("invalid value"),
        "error should describe enum variant mismatch: {message}"
    );
}
