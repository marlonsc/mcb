//! Error shape tests for MCP handler error responses.

use rstest::{fixture, rstest};
#[path = "handlers/test_helpers.rs"]
mod test_helpers;

use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource, SessionAction, SessionArgs};
use mcb_server::handlers::{MemoryHandler, SessionHandler};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use serde_json::{Value, json};

use test_helpers::{create_base_memory_args, create_real_domain_services};

fn error_text(result: &CallToolResult) -> String {
    serde_json::to_value(&result.content)
        .ok()
        .and_then(|value| value.as_array().cloned())
        .and_then(|items| items.first().cloned())
        .and_then(|item| item.get("text").cloned())
        .and_then(|text| text.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn assert_error_shape(result: &CallToolResult, expected_message: &str) {
    assert_eq!(result.is_error, Some(true));

    let content_json = serde_json::to_value(&result.content).expect("serialize content");
    assert!(content_json.is_array(), "error content must be an array");
    assert!(
        content_json
            .as_array()
            .is_some_and(|items| items.first().and_then(|item| item.get("text")).is_some()),
        "error content must contain a text field"
    );

    let text = error_text(result);
    assert!(
        text.contains(expected_message),
        "expected '{expected_message}' in '{text}'"
    );
}

async fn memory_handler() -> (MemoryHandler, tempfile::TempDir) {
    let (services, temp_dir) = create_real_domain_services().await;
    (MemoryHandler::new(services.memory_service), temp_dir)
}

async fn session_handler() -> (SessionHandler, tempfile::TempDir) {
    let (services, temp_dir) = create_real_domain_services().await;
    (
        SessionHandler::new(services.agent_session_service, services.memory_service),
        temp_dir,
    )
}

#[fixture]
fn observation_store_args() -> MemoryArgs {
    create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        None,
        None,
        Some("session-1".to_string()),
    )
}

#[rstest]
#[tokio::test]
async fn memory_store_missing_data_returns_expected_error(
    #[from(observation_store_args)] args: MemoryArgs,
) {
    let (handler, _temp_dir) = memory_handler().await;
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

    let (handler, _temp_dir) = memory_handler().await;
    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");

    assert_error_shape(&response, "Missing required field: content");
}

#[tokio::test]
async fn session_create_missing_data_returns_expected_error() {
    let (handler, _temp_dir) = session_handler().await;
    let args = SessionArgs {
        action: SessionAction::Create,
        org_id: None,
        session_id: None,
        data: None,
        project_id: None,
        worktree_id: None,
        parent_session_id: None,
        agent_type: Some("explore".to_string()),
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
