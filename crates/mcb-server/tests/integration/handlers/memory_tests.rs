use mcb_domain::value_objects::ids::SessionId;
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_server::handlers::MemoryHandler;
use rmcp::handler::server::wrapper::Parameters;

use rstest::*;
use serde_json::json;

use crate::utils::domain_services::create_base_memory_args;
use crate::utils::domain_services::create_real_domain_services;
use crate::utils::test_fixtures::{TEST_PROJECT_ID, TEST_SESSION_ID};

async fn create_handler() -> Option<(MemoryHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_real_domain_services().await?;
    Some((MemoryHandler::new(state.mcp_server.memory_service()), temp_dir))
}

fn missing_data_store_args() -> MemoryArgs {
    create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        None,
        None,
        None,
    )
}

fn get_missing_ids_args() -> MemoryArgs {
    MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: None,
        data: None,
        ids: None,
        repo_id: None,
        session_id: None,
        parent_session_id: None,
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: None,
    }
}

#[rstest]
#[tokio::test]
async fn test_memory_store_observation_success() {
    let Some((handler, _temp_dir)) = create_handler().await else {
        return;
    };
    let args = create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        Some(json!({
            "content": "Test observation",
            "observation_type": "code",
            "tags": ["test", "observation"]
        })),
        None,
        Some(TEST_SESSION_ID.to_owned()),
    );
    let mut args = args;
    args.project_id = Some(TEST_PROJECT_ID.to_owned());

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("memory handler should succeed for valid observation store");
    assert!(!response.content.is_empty(), "response should have content");
}

#[rstest]
#[case(missing_data_store_args())]
#[case(get_missing_ids_args())]
#[tokio::test]
async fn test_memory_validation_failures_return_error_response(#[case] args: MemoryArgs) {
    let Some((handler, _temp_dir)) = create_handler().await else {
        return;
    };
    let result = handler.handle(Parameters(args)).await;
    let response =
        result.expect("memory handler should return structured validation error response");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(response.is_error.unwrap_or(false));
}

#[rstest]
#[tokio::test]
async fn test_memory_inject_with_filters() {
    let Some((handler, _temp_dir)) = create_handler().await else {
        return;
    };
    let args = MemoryArgs {
        action: MemoryAction::Inject,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: Some(TEST_PROJECT_ID.to_owned()),
        data: None,
        ids: None,
        repo_id: Some("repo-123".to_owned()),
        session_id: None,
        parent_session_id: None,
        tags: Some(vec!["important".to_owned()]),
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: Some(1500),
        limit: Some(15),
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("memory handler should succeed for inject with filters");
    assert!(!response.content.is_empty(), "response should have content");
}

#[tokio::test]
async fn test_get_observations_missing_ids_returns_invalid_params() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };

    let args = MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: None,
        data: None,
        ids: None,
        repo_id: None,
        session_id: None,
        parent_session_id: None,
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("memory handler should return structured error response");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(response.is_error.unwrap_or(false), "Should return error");
}

#[tokio::test]
async fn test_store_session_missing_data_returns_invalid_params() {
    let Some((handler, _services_temp_dir)) = create_handler().await else {
        return;
    };

    let args = MemoryArgs {
        action: MemoryAction::Store,
        org_id: None,
        resource: MemoryResource::Session,
        project_id: None,
        data: None,
        ids: None,
        repo_id: None,
        session_id: Some(SessionId::from_name(TEST_SESSION_ID)),
        parent_session_id: None,
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: None,
    };

    let result = handler.handle(Parameters(args)).await;
    let response = result.expect("memory handler should return structured error response");
    assert!(!response.content.is_empty(), "response should have content");
    assert!(response.is_error.unwrap_or(false), "Should return error");
}
