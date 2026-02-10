//! Golden tests for memory and project workflow E2E validation.
//!
//! These tests verify the full stack integration for:
//! 1. Memory persistence (store, list, search)
//! 2. Project workflow (create, update, list phases/issues/dependencies)
//! 3. Context search (hybrid search across memory)

use mcb_server::args::{
    MemoryAction, MemoryArgs, MemoryResource, ProjectAction, ProjectArgs, ProjectResource,
    SearchArgs, SearchResource,
};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;

// Helper to extract text from response content
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
// Memory E2E Tests
// =============================================================================

#[tokio::test]
async fn test_golden_memory_store_with_default_project() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let memory_h = server.memory_handler();

    // Store observation with non-existent project (should auto-create default)
    let store_args = MemoryArgs {
        action: MemoryAction::Store,
        resource: MemoryResource::Observation,
        data: Some(json!({
            "content": "This is a test observation",
            "observation_type": "context",
            "tags": ["test", "golden"],
            "metadata": {
                "session_id": "test-session-1"
            }
        })),
        ids: None,
        project_id: Some("project-auto-create".to_string()),
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
    assert!(result.is_ok(), "memory store should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    // Response format is JSON with observation_id
    assert!(text.contains("observation_id"), "response: {}", text);
}

#[tokio::test]
async fn test_golden_memory_list_empty_graceful() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let memory_h = server.memory_handler();

    // List memories for a project with no data
    let list_args = MemoryArgs {
        action: MemoryAction::List,
        resource: MemoryResource::Observation,
        data: None,
        ids: None,
        project_id: Some("project-empty".to_string()),
        repo_id: None,
        session_id: None,
        tags: None,
        query: Some("missingterm".to_string()),
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
        "response: {}",
        text
    );
}

// =============================================================================
// Project Workflow E2E Tests
// =============================================================================

#[tokio::test]
async fn test_golden_project_create_phase() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let project_h = server.project_handler();
    let project_id = "test-project-1";

    // 1. Create Phase
    let create_args = ProjectArgs {
        action: ProjectAction::Create,
        resource: ProjectResource::Phase,
        project_id: project_id.to_string(),
        data: Some(json!({
            "name": "Phase 1: Design",
            "description": "Initial design phase"
        })),
        filters: None,
    };

    let result = project_h.handle(Parameters(create_args)).await;
    assert!(result.is_ok(), "create phase should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(text.contains("Created phase"), "response: {}", text);
}

#[tokio::test]
async fn test_golden_project_create_issue() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let project_h = server.project_handler();
    let project_id = "test-project-2";

    // 1. Create Issue
    let create_args = ProjectArgs {
        action: ProjectAction::Create,
        resource: ProjectResource::Issue,
        project_id: project_id.to_string(),
        data: Some(json!({
            "title": "Implement feature X",
            "description": "Details about feature X",
            "issue_type": "feature",
            "priority": 1,
            "labels": ["backend", "rust"]
        })),
        filters: None,
    };

    let result = project_h.handle(Parameters(create_args)).await;
    assert!(result.is_ok(), "create issue should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(text.contains("Created issue"), "response: {}", text);
}

#[tokio::test]
async fn test_golden_project_list_phases() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let project_h = server.project_handler();
    let project_id = "test-project-3";

    // Create a phase first
    project_h
        .handle(Parameters(ProjectArgs {
            action: ProjectAction::Create,
            resource: ProjectResource::Phase,
            project_id: project_id.to_string(),
            data: Some(json!({
                "name": "Phase A",
                "description": "Desc A"
            })),
            filters: None,
        }))
        .await
        .unwrap();

    // List phases
    let list_args = ProjectArgs {
        action: ProjectAction::List,
        resource: ProjectResource::Phase,
        project_id: project_id.to_string(),
        data: None,
        filters: None,
    };

    let result = project_h.handle(Parameters(list_args)).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(text.contains("Phase A"), "response: {}", text);
}

#[tokio::test]
async fn test_golden_project_update_issue_status() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let project_h = server.project_handler();
    let project_id = "test-project-4";

    // Create issue
    let create_resp = project_h
        .handle(Parameters(ProjectArgs {
            action: ProjectAction::Create,
            resource: ProjectResource::Issue,
            project_id: project_id.to_string(),
            data: Some(json!({
                "title": "Bug fix",
                "description": "Fix it",
                "issue_type": "bug",
                "priority": 0
            })),
            filters: None,
        }))
        .await
        .unwrap();

    // Extract ID from response (format: "Created issue: <uuid>")
    let create_text = extract_text(&create_resp.content);
    let issue_id = create_text.split_whitespace().last().unwrap();

    // Update status
    let update_args = ProjectArgs {
        action: ProjectAction::Update,
        resource: ProjectResource::Issue,
        project_id: project_id.to_string(),
        data: Some(json!({
            "id": issue_id,
            "status": "in_progress"
        })),
        filters: None,
    };

    let result = project_h.handle(Parameters(update_args)).await;
    assert!(result.is_ok(), "update issue should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(text.contains("Updated issue"), "response: {}", text);
}

#[tokio::test]
async fn test_golden_project_add_dependency() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let project_h = server.project_handler();
    let project_id = "test-project-5";

    // Create two issues
    let r1 = project_h
        .handle(Parameters(ProjectArgs {
            action: ProjectAction::Create,
            resource: ProjectResource::Issue,
            project_id: project_id.to_string(),
            data: Some(
                json!({ "title": "A", "description": "A", "issue_type": "task", "priority": 1 }),
            ),
            filters: None,
        }))
        .await
        .unwrap();
    let id1 = extract_text(&r1.content)
        .split_whitespace()
        .last()
        .unwrap()
        .to_string();

    let r2 = project_h
        .handle(Parameters(ProjectArgs {
            action: ProjectAction::Create,
            resource: ProjectResource::Issue,
            project_id: project_id.to_string(),
            data: Some(
                json!({ "title": "B", "description": "B", "issue_type": "task", "priority": 1 }),
            ),
            filters: None,
        }))
        .await
        .unwrap();
    let id2 = extract_text(&r2.content)
        .split_whitespace()
        .last()
        .unwrap()
        .to_string();

    // Add dependency: A blocks B
    let dep_args = ProjectArgs {
        action: ProjectAction::Create,
        resource: ProjectResource::Dependency,
        project_id: project_id.to_string(),
        data: Some(json!({
            "from_issue_id": id1,
            "to_issue_id": id2,
            "dependency_type": "blocks"
        })),
        filters: None,
    };

    let result = project_h.handle(Parameters(dep_args)).await;
    assert!(result.is_ok(), "add dependency should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(text.contains("Added dependency"), "response: {}", text);
}

// =============================================================================
// Context Search E2E Tests
// =============================================================================

#[tokio::test]
async fn test_golden_context_search_basic() {
    let (server, _temp) = crate::test_utils::test_fixtures::create_test_mcp_server().await;
    let memory_h = server.memory_handler();
    let search_h = server.search_handler();
    let project_id = "search-project";

    // 1. Store context observations
    let _ = memory_h
        .handle(Parameters(MemoryArgs {
            action: MemoryAction::Store,
            resource: MemoryResource::Observation,
            data: Some(json!({
                "content": "The reactor core temperature is critical.",
                "observation_type": "context",
                "tags": ["reactor", "critical"],
                "metadata": { "session_id": "s1" }
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
        }))
        .await;

    // 2. Search using Context resource
    let search_args = SearchArgs {
        query: "reactor temperature".to_string(),
        resource: SearchResource::Context,
        collection: None,
        extensions: None,
        filters: None,
        limit: Some(5),
        min_score: None,
        tags: None,
        session_id: None,
        token: None,
    };

    let result = search_h.handle(Parameters(search_args)).await;
    assert!(result.is_ok(), "context search should succeed");
    let resp = result.unwrap();
    let text = extract_text(&resp.content);
    assert!(
        text.contains("reactor core temperature"),
        "Search results missing content: {}",
        text
    );
}
