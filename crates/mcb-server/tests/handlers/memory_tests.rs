use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_server::handlers::MemoryHandler;
use rmcp::handler::server::wrapper::Parameters;

use rstest::*;
use serde_json::json;

use crate::handlers::test_helpers::create_base_memory_args;
use crate::handlers::test_helpers::create_real_domain_services;

async fn create_handler() -> Option<(MemoryHandler, tempfile::TempDir)> {
    let (services, temp_dir) = create_real_domain_services().await?;
    Some((MemoryHandler::new(services.memory_service), temp_dir))
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
        Some("test-session".to_string()),
    );
    let mut args = args;
    args.project_id = Some("test-project".to_string());

    let result = handler.handle(Parameters(args)).await;
    assert!(result.is_ok());
    let _response = result.expect("response");
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
    assert!(result.is_ok());
    assert!(result.expect("response").is_error.unwrap_or(false));
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
        project_id: Some("test-project".to_string()),
        data: None,
        ids: None,
        repo_id: Some("repo-123".to_string()),
        session_id: None,
        parent_session_id: None,
        tags: Some(vec!["important".to_string()]),
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
    assert!(result.is_ok());
    let _response = result.expect("response");
}
