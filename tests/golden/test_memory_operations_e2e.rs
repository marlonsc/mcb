/// Golden tests: Session memory operations with auto-hooking
/// Verifies store_observation, timeline, search, and hook integration
use mcb_domain::utils::tests::fixtures::create_test_mcp_server;
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_store_observation() {
    let server = create_test_mcp_server().await;

    let result = server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(serde_json::json!({
                "observation_type": "Decision",
                "content": "Chosen auth approach: JWT with refresh tokens",
                "tags": ["auth", "security"],
                "project_id": "test-project"
            })),
            ids: None,
            project_id: Some("test-project".to_string()),
            repo_id: None,
            session_id: None,
            tags: Some(vec!["auth".to_string()]),
            query: None,
            anchor_id: None,
            depth_before: None,
            depth_after: None,
            window_secs: None,
            observation_types: None,
            max_tokens: None,
            limit: None,
        }))
        .await;

    assert!(result.is_ok(), "Store observation should succeed");
    let response = result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Response should not be error"
    );
}

#[tokio::test]
async fn golden_search_memory() {
    let server = create_test_mcp_server().await;

    server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(serde_json::json!({
                "observation_type": "Decision",
                "content": "JWT auth implementation decision",
                "tags": ["auth"],
                "project_id": "test-project"
            })),
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
        }))
        .await
        .expect("store");

    let search_result = server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Search,
            resource: MemoryResource::Observation,
            data: None,
            ids: None,
            project_id: Some("test-project".to_string()),
            repo_id: None,
            session_id: None,
            tags: Some(vec!["auth".to_string()]),
            query: Some("JWT authentication".to_string()),
            anchor_id: None,
            depth_before: None,
            depth_after: None,
            window_secs: None,
            observation_types: Some(vec!["Decision".to_string()]),
            max_tokens: None,
            limit: Some(10),
        }))
        .await;

    assert!(search_result.is_ok(), "Memory search should succeed");
    let response = search_result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Search response should not be error"
    );
}

#[tokio::test]
async fn golden_timeline_memory() {
    let server = create_test_mcp_server().await;

    server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(serde_json::json!({
                "observation_type": "Decision",
                "content": "Test decision 1",
                "tags": ["test"],
                "project_id": "test-project"
            })),
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
        }))
        .await
        .expect("store first");

    let timeline_result = server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Timeline,
            resource: MemoryResource::Observation,
            data: None,
            ids: None,
            project_id: Some("test-project".to_string()),
            repo_id: None,
            session_id: None,
            tags: None,
            query: None,
            anchor_id: None,
            depth_before: Some(5),
            depth_after: Some(5),
            window_secs: Some(3600),
            observation_types: None,
            max_tokens: None,
            limit: Some(20),
        }))
        .await;

    assert!(timeline_result.is_ok(), "Timeline query should succeed");
}

#[tokio::test]
async fn golden_get_observations() {
    let server = create_test_mcp_server().await;

    let store_result = server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(serde_json::json!({
                "observation_type": "Execution",
                "content": "Deployment executed",
                "tags": ["deploy"],
                "project_id": "test-project"
            })),
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
        }))
        .await;

    assert!(store_result.is_ok(), "Store should succeed");

    let get_result = server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Get,
            resource: MemoryResource::Observation,
            data: None,
            ids: Some(vec!["test-obs-1".to_string()]),
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
        }))
        .await;

    assert!(get_result.is_ok(), "Get observations should succeed");
}

#[tokio::test]
async fn golden_inject_context_at_session_start() {
    let server = create_test_mcp_server().await;

    server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(serde_json::json!({
                "observation_type": "Summary",
                "content": "Session summary with context",
                "tags": ["summary"],
                "project_id": "test-project"
            })),
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
        }))
        .await
        .expect("store");

    let inject_result = server
        .memory_handler()
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Inject,
            resource: MemoryResource::Observation,
            data: None,
            ids: None,
            project_id: Some("test-project".to_string()),
            repo_id: None,
            session_id: Some("test-session-123".to_string()),
            tags: None,
            query: None,
            anchor_id: None,
            depth_before: Some(3),
            depth_after: Some(3),
            window_secs: Some(86400),
            observation_types: None,
            max_tokens: Some(2000),
            limit: None,
        }))
        .await;

    assert!(inject_result.is_ok(), "Context injection should succeed");
    let response = inject_result.unwrap();
    assert!(
        !response.is_error.unwrap_or(true),
        "Injection response should not be error"
    );
}
