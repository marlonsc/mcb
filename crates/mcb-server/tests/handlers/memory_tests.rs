use std::sync::Arc;

use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_server::handlers::MemoryHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::handlers::test_helpers::create_base_memory_args;
use crate::test_utils::mock_services::MockMemoryService;

#[tokio::test]
async fn test_memory_store_observation_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        Some(json!({
            "content": "Test observation",
            "observation_type": "discovery",
            "tags": ["test", "observation"]
        })),
        None,
        Some("test-session".to_string()),
    );

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let _response = result.expect("Expected successful response");
}

#[tokio::test]
async fn test_memory_store_observation_missing_data() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        None,
        None,
        None,
    );

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing data should return error"
    );
}

#[tokio::test]
async fn test_memory_store_execution_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Store,
        org_id: None,
        resource: MemoryResource::Execution,
        project_id: Some("test-project".to_string()),
        data: Some(json!({
            "command": "test command",
            "exit_code": 0,
            "duration_ms": 1000,
            "success": true,
            "execution_type": "build"
        })),
        ids: None,
        repo_id: None,
        session_id: Some("test-session".to_string().into()),
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

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_store_quality_gate_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Store,
        org_id: None,
        resource: MemoryResource::QualityGate,
        project_id: Some("test-project".to_string()),
        data: Some(json!({
            "gate_name": "test_gate",
            "status": "passed",
            "metrics": {}
        })),
        ids: None,

        repo_id: None,
        session_id: Some("test-session".to_string().into()),
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

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_store_session_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Store,
        org_id: None,
        resource: MemoryResource::Session,
        project_id: Some("test-project".to_string()),
        data: Some(json!({
            "session_id": "test-session",
            "summary": "Test session summary"
        })),
        ids: None,
        repo_id: None,
        session_id: Some("test-session".to_string().into()),
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

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_get_observation_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: None,
        data: None,
        ids: Some(vec!["obs-1".to_string(), "obs-2".to_string()]),
        repo_id: None,
        session_id: None,
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

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_get_observation_missing_ids() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: None,
        data: None,
        ids: None,
        repo_id: None,
        session_id: None,
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

    assert!(result.is_ok());
    let response = result.expect("Expected response");
    assert!(
        response.is_error.unwrap_or(false),
        "Missing ids should return error"
    );
}

#[tokio::test]
async fn test_memory_get_execution_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::Execution,
        project_id: None,
        data: None,
        ids: Some(vec!["exec-1".to_string()]),
        repo_id: None,
        session_id: None,
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

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_get_quality_gate_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::QualityGate,
        project_id: None,
        data: None,
        ids: Some(vec!["qg-1".to_string()]),
        repo_id: None,
        session_id: None,
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

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_get_session_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Get,
        org_id: None,
        resource: MemoryResource::Session,
        project_id: Some("test-project".to_string()),
        data: None,
        ids: None,
        repo_id: None,
        session_id: Some("test-session".to_string().into()),
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

    assert!(result.is_ok());
    let _response = result.expect("Expected successful response");
}

#[tokio::test]
async fn test_memory_inject_with_filters() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Inject,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: None,
        data: None,
        ids: None,
        repo_id: Some("repo-123".to_string()),
        session_id: None,
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
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}
