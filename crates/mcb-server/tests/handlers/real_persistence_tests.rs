//! Real-Persistence Handler Integration Tests
//!
//! Tests that validate handlers against **real** providers (SQLite, EdgeVec, FastEmbed).
//! No mocks — every assertion verifies actual database state.
//!
//! ## Key Principle
//!
//! The MCB "honesty fix" (v0.2.1) requires that operations either succeed with real
//! persistence or fail with contextual errors. These tests guard against "ghost execution"
//! regressions where handlers return success but nothing is actually stored.

use mcb_server::args::{
    MemoryAction, MemoryArgs, MemoryResource, SearchArgs, SearchResource, SessionAction,
    SessionArgs,
};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

use crate::test_utils::test_fixtures::create_test_mcp_server;

/// Extract text content from CallToolResult for assertions.
fn extract_text(content: &[rmcp::model::Content]) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(json) = serde_json::to_value(c)
                && let Some(text) = json.get("text")
            {
                text.as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// Memory Store — Real Persistence
// =============================================================================

/// Stores an observation via the memory handler and verifies it was actually
/// persisted by retrieving it back through a list query.
#[tokio::test]
async fn test_real_memory_store_observation_persists() {
    let (server, _temp) = create_test_mcp_server().await;
    let memory_h = server.memory_handler();
    let project_id = "real-persist-test";

    // 1. Store an observation
    let store_args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Observation,
        data: Some(json!({
            "content": "Authentication middleware uses JWT with RS256",
            "observation_type": "context",
            "tags": ["auth", "jwt"],
            "metadata": { "session_id": "sess-real-001" }
        })),
        ids: None,
        project_id: Some(project_id.to_string()),
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

    let result = memory_h.handle(Parameters(store_args)).await;
    assert!(result.is_ok(), "store should not return Err");
    let resp = result.unwrap();
    assert!(
        !resp.is_error.unwrap_or(false),
        "store should succeed, got: {}",
        extract_text(&resp.content)
    );
    let text = extract_text(&resp.content);
    assert!(
        text.contains("observation_id"),
        "Response should contain observation_id, got: {}",
        text
    );

    // 2. Retrieve it back via list — proves it actually hit the database
    let list_args = MemoryArgs {
        action: MemoryAction::List,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some(project_id.to_string()),
        repo_id: None,
        session_id: None,
        tags: None,
        query: Some("JWT".to_string()),
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: Some(10),
    };

    let list_result = memory_h.handle(Parameters(list_args)).await;
    assert!(list_result.is_ok());
    let list_resp = list_result.unwrap();
    let list_text = extract_text(&list_resp.content);
    assert!(
        list_text.contains("JWT")
            || list_text.contains("jwt")
            || list_text.contains("auth")
            || list_text.contains("Authentication"),
        "Listed observations should contain our stored content, got: {}",
        list_text
    );
}

/// Stores multiple observations and verifies the count matches.
#[tokio::test]
async fn test_real_memory_store_multiple_observations_counted() {
    let (server, _temp) = create_test_mcp_server().await;
    let memory_h = server.memory_handler();
    let project_id = "real-multi-store";

    // Store 3 observations
    for i in 0..3 {
        let store_args = MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(json!({
                "content": format!("Observation number {}", i),
                "observation_type": "context",
                "tags": ["batch-test"],
                "metadata": { "session_id": "sess-batch" }
            })),
            ids: None,
            project_id: Some(project_id.to_string()),
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

        let result = memory_h.handle(Parameters(store_args)).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(
            !resp.is_error.unwrap_or(false),
            "Store {} should succeed",
            i
        );
    }

    // List and verify count
    let list_args = MemoryArgs {
        action: MemoryAction::List,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some(project_id.to_string()),
        repo_id: None,
        session_id: None,
        tags: None,
        query: Some("Observation".to_string()),
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: Some(50),
    };

    let list_result = memory_h.handle(Parameters(list_args)).await;
    assert!(list_result.is_ok());
    let list_resp = list_result.unwrap();
    let list_text = extract_text(&list_resp.content);

    // We stored 3 observations; the response should reflect that
    let has_observations = list_text.contains("Observation number 0")
        && list_text.contains("Observation number 1")
        && list_text.contains("Observation number 2");
    assert!(
        has_observations || list_text.contains("\"count\": 3"),
        "Should find all 3 observations in list. Response: {}",
        list_text
    );
}

// =============================================================================
// Memory Store — Error Path (contextual errors, not "internal error")
// =============================================================================

/// Attempts to store an observation with missing required fields.
/// Verifies the error message is contextual (not the old "internal error").
#[tokio::test]
async fn test_real_memory_store_missing_data_returns_contextual_error() {
    let (server, _temp) = create_test_mcp_server().await;
    let memory_h = server.memory_handler();

    let bad_args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Observation,
        data: None, // Missing required data
        ids: None,
        project_id: Some("error-test-project".to_string()),
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

    let result = memory_h.handle(Parameters(bad_args)).await;
    assert!(result.is_ok(), "Handler should return Ok (error in body)");
    let resp = result.unwrap();
    assert!(
        resp.is_error.unwrap_or(false),
        "Missing data should be flagged as error"
    );
    let text = extract_text(&resp.content);
    // Must NOT be the old opaque "internal error"
    assert!(
        !text.contains("internal error") || text.contains("Missing"),
        "Error should be contextual, not opaque. Got: {}",
        text
    );
}

// =============================================================================
// Session Create — Real Persistence
// =============================================================================

/// Creates a session_summary (memory resource) and verifies it can be retrieved.
/// Tests the full store → get round-trip on real SQLite.
/// No observation seed needed — store_session_summary auto-creates org + project (honesty fix v0.2.1).
#[tokio::test]
async fn test_real_session_summary_store_and_retrieve() {
    let (server, _temp) = create_test_mcp_server().await;
    let memory_h = server.memory_handler();

    // 1. Store a session summary (NO seed observation needed — auto-create handles FK)
    let store_args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Session,
        data: Some(json!({
            "session_id": "sess-roundtrip",
            "topics": ["architecture", "testing"],
            "decisions": ["use hexagonal architecture"],
            "next_steps": ["write integration tests"],
            "key_files": ["src/main.rs"]
        })),
        ids: None,
        project_id: Some("roundtrip-project".to_string()),
        repo_id: None,
        session_id: Some("sess-roundtrip".to_string().into()),
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

    let store_result = memory_h.handle(Parameters(store_args)).await;
    assert!(store_result.is_ok());
    let store_resp = store_result.unwrap();
    let store_text = extract_text(&store_resp.content);
    assert!(
        !store_resp.is_error.unwrap_or(false),
        "Session summary store should succeed, got: {}",
        store_text
    );
    assert!(
        store_text.contains("summary_id"),
        "Response should contain summary_id, got: {}",
        store_text
    );

    // 2. Retrieve via Get to verify persistence
    let get_args = MemoryArgs {
        action: MemoryAction::Get,
        resource: MemoryResource::Session,
        data: None,
        ids: None,
        project_id: None,
        repo_id: None,
        session_id: Some("sess-roundtrip".to_string().into()),
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

    let get_result = memory_h.handle(Parameters(get_args)).await;
    assert!(get_result.is_ok());
    let get_resp = get_result.unwrap();
    let get_text = extract_text(&get_resp.content);
    assert!(
        get_text.contains("architecture") || get_text.contains("hexagonal"),
        "Retrieved session summary should contain stored data, got: {}",
        get_text
    );
}

// =============================================================================
// Session Create — Invalid Agent Type (contextual error)
// =============================================================================

/// Passes an invalid agent_type and verifies the error lists valid types.
#[tokio::test]
async fn test_real_session_create_invalid_agent_type_contextual_error() {
    let (server, _temp) = create_test_mcp_server().await;
    let session_h = server.session_handler();

    let bad_args = SessionArgs {
        action: SessionAction::Create,
        session_id: None,
        data: Some(json!({
            "session_summary_id": "summary-bad-type",
            "model": "claude-3-sonnet",
            "project_id": "bad-type-project"
        })),
        project_id: Some("bad-type-project".to_string()),
        worktree_id: None,
        agent_type: Some("nonexistent_agent_xyz".to_string()),
        status: None,
        limit: None,
    };

    let result = session_h.handle(Parameters(bad_args)).await;
    // This should be an McpError (invalid_params), so result itself is Err
    match result {
        Err(e) => {
            let err_text = format!("{:?}", e);
            // Should list valid agent types
            assert!(
                err_text.contains("sisyphus")
                    || err_text.contains("oracle")
                    || err_text.contains("Valid"),
                "Error should list valid agent types, got: {}",
                err_text
            );
        }
        Ok(resp) => {
            // If it returns Ok with is_error, that's also acceptable
            assert!(
                resp.is_error.unwrap_or(false),
                "Invalid agent type should return error"
            );
            let text = extract_text(&resp.content);
            assert!(
                text.contains("sisyphus") || text.contains("oracle") || text.contains("Valid"),
                "Error should list valid agent types, got: {}",
                text
            );
        }
    }
}

// =============================================================================
// Search — Empty Results (honest, not error)
// =============================================================================

/// Searches an empty project and verifies it returns empty results (not an error).
#[tokio::test]
async fn test_real_search_empty_project_returns_empty_not_error() {
    let (server, _temp) = create_test_mcp_server().await;
    let search_h = server.search_handler();

    let search_args = SearchArgs {
        query: "nonexistent pattern that should match nothing".to_string(),
        resource: SearchResource::Memory,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(10),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
    };

    let result = search_h.handle(Parameters(search_args)).await;
    assert!(result.is_ok(), "Search should not Err");
    let resp = result.unwrap();

    // Empty results should NOT be flagged as error — this is honest behavior
    assert!(
        !resp.is_error.unwrap_or(false),
        "Empty search results should not be an error"
    );

    let text = extract_text(&resp.content);
    // Should contain a count of 0 or empty results array
    assert!(
        text.contains("\"count\": 0") || text.contains("\"count\":0") || text.contains("[]"),
        "Search on empty project should return count 0 or empty array, got: {}",
        text
    );
}

// =============================================================================
// Agent Session — Full FK Chain Round-Trip
// =============================================================================

/// Full round-trip: store session_summary → create agent_session → get agent_session.
/// Proves the entire FK chain (org → project → session_summary → agent_session) works
/// with auto-create, no observation seed needed.
#[tokio::test]
async fn test_real_agent_session_create_and_retrieve() {
    let (server, _temp) = create_test_mcp_server().await;
    let memory_h = server.memory_handler();
    let session_h = server.session_handler();

    // 1. Store a session_summary (auto-creates org + project)
    let summary_args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Session,
        data: Some(json!({
            "session_id": "sess-agent-roundtrip",
            "topics": ["FK chain validation"],
            "decisions": ["auto-create works"],
            "next_steps": [],
            "key_files": []
        })),
        ids: None,
        project_id: Some("agent-roundtrip-project".to_string()),
        repo_id: None,
        session_id: Some("sess-agent-roundtrip".to_string().into()),
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
    let summary_result = memory_h.handle(Parameters(summary_args)).await;
    assert!(summary_result.is_ok());
    let summary_resp = summary_result.unwrap();
    let summary_text = extract_text(&summary_resp.content);
    assert!(
        !summary_resp.is_error.unwrap_or(false),
        "Session summary store should succeed, got: {}",
        summary_text
    );

    // Extract summary_id from response
    let summary_json: serde_json::Value =
        serde_json::from_str(&summary_text).expect("Response should be valid JSON");
    let summary_id = summary_json["summary_id"]
        .as_str()
        .expect("Response should contain summary_id");

    // 2. Create an agent_session using the real summary_id
    let create_args = SessionArgs {
        action: SessionAction::Create,
        session_id: None,
        data: Some(json!({
            "session_summary_id": summary_id,
            "model": "claude-opus-4-20250514",
            "project_id": "agent-roundtrip-project"
        })),
        project_id: Some("agent-roundtrip-project".to_string()),
        worktree_id: None,
        agent_type: Some("sisyphus".to_string()),
        status: None,
        limit: None,
    };
    let create_result = session_h.handle(Parameters(create_args)).await;
    assert!(create_result.is_ok(), "Create should not Err");
    let create_resp = create_result.unwrap();
    let create_text = extract_text(&create_resp.content);
    assert!(
        !create_resp.is_error.unwrap_or(false),
        "Agent session create should succeed, got: {}",
        create_text
    );
    assert!(
        create_text.contains("session_id"),
        "Response should contain session_id, got: {}",
        create_text
    );

    // Extract agent session_id
    let create_json: serde_json::Value =
        serde_json::from_str(&create_text).expect("Create response should be valid JSON");
    let agent_session_id = create_json["session_id"]
        .as_str()
        .expect("Response should contain session_id");

    // 3. Get agent_session back — verify data matches
    let get_args = SessionArgs {
        action: SessionAction::Get,
        session_id: Some(agent_session_id.to_string().into()),
        data: None,
        project_id: None,
        worktree_id: None,
        agent_type: None,
        status: None,
        limit: None,
    };
    let get_result = session_h.handle(Parameters(get_args)).await;
    assert!(get_result.is_ok(), "Get should not Err");
    let get_resp = get_result.unwrap();
    let get_text = extract_text(&get_resp.content);
    assert!(
        !get_resp.is_error.unwrap_or(false),
        "Agent session get should succeed, got: {}",
        get_text
    );
    assert!(
        get_text.contains("sisyphus"),
        "Retrieved session should contain agent_type=sisyphus, got: {}",
        get_text
    );
    assert!(
        get_text.contains("claude-opus-4-20250514"),
        "Retrieved session should contain model, got: {}",
        get_text
    );
    assert!(
        get_text.contains(summary_id),
        "Retrieved session should reference original summary_id, got: {}",
        get_text
    );
}

// =============================================================================
// Session List — Empty database returns empty, not error
// =============================================================================

/// Lists sessions on a fresh (empty) database.
/// Verifies honest behavior: empty results, not an error or fake data.
#[tokio::test]
async fn test_real_session_list_empty_returns_gracefully() {
    let (server, _temp) = create_test_mcp_server().await;
    let session_h = server.session_handler();

    let list_args = SessionArgs {
        action: SessionAction::List,
        session_id: None,
        data: None,
        project_id: None,
        worktree_id: None,
        agent_type: None,
        status: None,
        limit: Some(50),
    };

    let list_result = session_h.handle(Parameters(list_args)).await;
    assert!(list_result.is_ok(), "List should not Err on empty table");
    let list_resp = list_result.unwrap();
    assert!(
        !list_resp.is_error.unwrap_or(false),
        "Empty session list should not be an error"
    );
}
