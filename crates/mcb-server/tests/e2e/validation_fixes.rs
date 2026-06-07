//! Golden tests for v0.2.0 validation fixes.
//!
//! Covers:
//! 1. Agent SQL Storage (`log_tool`) - P1
//! 2. Session Create Schema Fallback (`agent_type` in data) - P2
//! 3. Memory Observation Enum Validation - P2

use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::utils::time::epoch_secs_i64;
use mcb_server::args::{
    AgentAction, AgentArgs, MemoryAction, MemoryArgs, MemoryResource, SessionAction, SessionArgs,
};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::utils::test_fixtures::TEST_PROJECT_ID;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_validation_agent_sql_storage_flow() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;

    // Create a session first — LogTool requires an existing session in the DB
    let session_id_str = "test-sql-storage-session";
    let now = epoch_secs_i64().unwrap_or(0);
    let session = AgentSession {
        id: mcb_domain::value_objects::SessionId::from_name(session_id_str).to_string(),
        session_summary_id: "test-summary".to_owned(),
        agent_type: AgentType::Sisyphus,
        model: "test-model".to_owned(),
        parent_session_id: None,
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: None,
        result_summary: None,
        token_count: None,
        tool_calls_count: None,
        delegations_count: None,
        project_id: None,
        worktree_id: None,
    };
    server
        .agent_session_service()
        .create_session(session)
        .await
        .expect("pre-create agent session");

    let agent_h = server.agent_handler();
    let result = agent_h
        .handle(Parameters(AgentArgs {
            action: AgentAction::LogTool,
            org_id: None,
            session_id: Some(mcb_domain::value_objects::SessionId::from_name(
                session_id_str,
            )),
            data: json!({
                "tool_name": "test",
                "success": true,
                "duration_ms": 100
            }),
        }))
        .await;

    let resp = result.expect("agent log_tool should return a response");
    assert!(!resp.content.is_empty(), "response should have content");
    let text = extract_text(&resp.content);
    assert!(
        text.contains("tool_call_id") && text.contains("tool_name"),
        "LogTool should succeed and return tool_call_id. Got: {text}"
    );
    assert!(!resp.is_error.unwrap_or(false));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_validation_session_create_schema_fallback() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
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
                "project_id": TEST_PROJECT_ID
            })),
            worktree_id: None,
            parent_session_id: None,
            session_id: None,
            project_id: Some(TEST_PROJECT_ID.to_owned()),
            limit: None,
            status: None,
        }))
        .await;

    // Should NOT fail with "Missing agent_type"
    let resp = result.expect("session create with fallback agent_type should return a response");
    assert!(!resp.content.is_empty(), "response should have content");
    let text = extract_text(&resp.content);
    assert!(
        !text.contains("Missing agent_type"),
        "Fallback failed: {text}"
    );
    // It will likely fail with "Failed to create agent session" due to missing FK, which is fine
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_validation_memory_observation_enum_error() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
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

    let resp = result.expect("invalid observation_type should return an error response");
    assert!(
        !resp.content.is_empty(),
        "error response should have content"
    );
    assert!(resp.is_error.unwrap_or(false));
    let text = extract_text(&resp.content);
    assert!(
        text.contains("Invalid observation_type:") || text.contains("Unknown observation type:"),
        "Error message validation failed: {text}"
    );
    Ok(())
}
