use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_server::handlers::MemoryHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;
use std::sync::Arc;

use crate::test_utils::mock_services::MockMemoryService;

#[tokio::test]
async fn test_memory_store_observation_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Observation,
        data: Some(json!({
            "content": "Test observation",
            "type": "discovery",
            "tags": ["test", "observation"]
        })),
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
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
async fn test_memory_store_observation_missing_data() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
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
        "Missing data should return error"
    );
}

#[tokio::test]
async fn test_memory_store_execution_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Execution,
        data: Some(json!({
            "command": "test command",
            "status": "success",
            "output": "test output"
        })),
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
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
        resource: MemoryResource::QualityGate,
        data: Some(json!({
            "gate_name": "test_gate",
            "status": "passed",
            "metrics": {}
        })),
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
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
        resource: MemoryResource::Session,
        data: Some(json!({
            "session_id": "test-session",
            "summary": "Test session summary"
        })),
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
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
        resource: MemoryResource::Observation,
        data: None,
        ids: Some(vec!["obs-1".to_string(), "obs-2".to_string()]),
        project_id: Some("test-project".to_string()),
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
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
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
        resource: MemoryResource::Execution,
        data: None,
        ids: Some(vec!["exec-1".to_string()]),
        project_id: Some("test-project".to_string()),
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
        resource: MemoryResource::QualityGate,
        data: None,
        ids: Some(vec!["qg-1".to_string()]),
        project_id: Some("test-project".to_string()),
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
        resource: MemoryResource::Session,
        data: None,
        ids: Some(vec!["sess-1".to_string()]),
        project_id: Some("test-project".to_string()),
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
async fn test_memory_list_observations_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::List,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
        tags: Some(vec!["test".to_string()]),
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: Some(10),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_list_non_observation_resource_fails() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::List,
        resource: MemoryResource::Execution,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
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
        "List only supports observations"
    );
}

#[tokio::test]
async fn test_memory_timeline_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Timeline,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
        tags: None,
        query: None,
        anchor_id: Some("obs-anchor".to_string()),
        depth_before: Some(5),
        depth_after: Some(5),
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
async fn test_memory_timeline_missing_anchor() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Timeline,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: Some(5),
        depth_after: Some(5),
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
        "Missing anchor_id should return error"
    );
}

#[tokio::test]
async fn test_memory_inject_success() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Inject,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: Some("test-session".to_string()),
        tags: Some(vec!["important".to_string()]),
        query: Some("test query".to_string()),
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: Some(vec!["discovery".to_string()]),
        max_tokens: Some(2000),
        limit: Some(20),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_memory_inject_missing_session() {
    let mock_service = MockMemoryService::new();
    let handler = MemoryHandler::new(Arc::new(mock_service));

    let args = MemoryArgs {
        action: MemoryAction::Inject,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: None,
        tags: None,
        query: Some("test query".to_string()),
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
        "Missing session_id should return error"
    );
}
