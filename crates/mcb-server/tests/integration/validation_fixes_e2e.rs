//! Golden tests for v0.2.0 validation fixes.
//!
//! Covers:
//! 1. Agent SQL Storage (`log_tool`) - P1
//! 2. Session Create Schema Fallback (`agent_type` in data) - P2
//! 3. Memory Observation Enum Validation - P2

use mcb_server::args::{
    AgentAction, AgentArgs, MemoryAction, MemoryArgs, MemoryResource, SessionAction, SessionArgs,
};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::test_utils::text::extract_text;

#[tokio::test]
async fn test_validation_agent_sql_storage_flow() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let agent_h = server.agent_handler();

    // Verify LogTool handles repo errors gracefully (proving handler execution)
    let result = agent_h
        .handle(Parameters(AgentArgs {
            action: AgentAction::LogTool,
            org_id: None,
            session_id: mcb_domain::value_objects::SessionId::from_name("missing-session"),
            data: json!({
                "tool_name": "test",
                "success": true,
                "duration_ms": 100
            }),
        }))
        .await;

    // Should return a handled error, not panic or internal server error
    assert!(result.is_ok());
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(
        text.contains("Memory storage error"),
        "Should handle SQL error gracefully. Got: {text}"
    );
    assert!(resp.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_validation_session_create_schema_fallback() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let session_h = server.session_handler();

    // Try creating session with agent_type inside data (MISSING from top-level args)
    let result = session_h
        .handle(Parameters(SessionArgs {
            action: SessionAction::Create,
            org_id: None,
            agent_type: None, // MISSING
            data: Some(json!({
                "session_summary_id": "summ-1",
                "model": "test-model",
                "agent_type": "sisyphus", // FALLBACK
                "project_id": "test-project"
            })),
            worktree_id: None,
            parent_session_id: None,
            session_id: None,
            project_id: Some("test-project".to_owned()),
            limit: None,
            status: None,
        }))
        .await;

    // Should NOT fail with "Missing agent_type"
    assert!(result.is_ok());
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(
        !text.contains("Missing agent_type"),
        "Fallback failed: {text}"
    );
    // It will likely fail with "Failed to create agent session" due to missing FK, which is fine
}

#[tokio::test]
async fn test_validation_memory_observation_enum_error() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let memory_h = server.memory_handler();

    let result = memory_h
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            org_id: None,
            resource: MemoryResource::Observation,
            project_id: None,
            data: Some(json!({
                "content": "test",
                "observation_type": "INVALID_TYPE"
            })),
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
        }))
        .await;

    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(resp.is_error.unwrap_or(false));
    let text = extract_text(&resp.content);
    assert!(
        text.contains("Unknown observation type:"),
        "Error message validation failed: {text}"
    );
}
