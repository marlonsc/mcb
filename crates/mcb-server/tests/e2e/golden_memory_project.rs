//! Golden tests for memory and context search E2E validation.
//!
//! These tests verify the full stack integration for:
//! 1. Memory persistence (store, list, search)
//! 2. Context search (hybrid search across memory)

use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource, SearchArgs, SearchResource};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::utils::text::extract_text;
use mcb_domain::test_utils::TestResult;
use rstest::rstest;

// =============================================================================
// Memory E2E Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_golden_memory_store_with_default_project() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let memory_h = server.memory_handler();

    // Store observation with a test project
    let store_args = MemoryArgs {
        action: MemoryAction::Store,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: Some("golden-test-project".to_owned()),
        data: Some(json!({
            "content": "This is a test observation",
            "observation_type": "context",
            "tags": ["test", "golden"],
            "metadata": {
                "session_id": "test-session-1"
            }
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
    };

    let result = memory_h.handle(Parameters(store_args)).await;
    assert!(result.is_ok(), "memory store should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    // Response format is JSON with observation_id
    assert!(text.contains("observation_id"), "response: {text}");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_golden_memory_list_empty_graceful() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let memory_h = server.memory_handler();

    // List memories for a project with no data
    let list_args = MemoryArgs {
        action: MemoryAction::List,
        org_id: None,
        resource: MemoryResource::Observation,
        project_id: None,
        data: None,
        ids: None,
        repo_id: None,
        session_id: None,
        parent_session_id: None,
        tags: None,
        query: Some("missingterm".to_owned()),
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: Some(10),
    };

    let result = memory_h.handle(Parameters(list_args)).await;
    assert!(result.is_ok(), "memory list should succeed (empty result)");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    // Should return valid JSON with empty results, not error
    assert!(
        text.contains("\"count\": 0") || text.contains("[]"),
        "response: {text}"
    );
    Ok(())
}

// =============================================================================
// Context Search E2E Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_golden_context_search_basic() -> TestResult {
    let (server, _temp) = crate::utils::test_fixtures::create_test_mcp_server().await?;
    let memory_h = server.memory_handler();
    let search_h = server.search_handler();
    let project_id = "search-project";

    // 1. Store context observations
    let _ = memory_h
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            org_id: None,
            resource: MemoryResource::Observation,
            project_id: Some(project_id.to_owned()),
            data: Some(json!({
                "content": "The reactor core temperature is critical.",
                "observation_type": "context",
                "tags": ["reactor", "critical"],
                "metadata": { "session_id": "s1" }
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

    // 2. Search using Context resource
    let search_args = SearchArgs {
        query: "reactor temperature".to_owned(),
        org_id: None,
        resource: SearchResource::Context,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(5),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
        repo_id: None,
    };

    let result = search_h.handle(Parameters(search_args)).await;
    assert!(result.is_ok(), "context search should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(
        text.contains("reactor core temperature"),
        "Search results missing content: {text}"
    );
    Ok(())
}
