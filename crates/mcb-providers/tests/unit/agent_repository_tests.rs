//! Unit tests for SqliteAgentRepository
//!
//! Tests cover creation and storage of agent sessions and tool calls, verifying
//! SQL schema integration and foreign key constraints.

use std::sync::Arc;

use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, ToolCall};
use mcb_domain::entities::memory::SessionSummary;
use mcb_domain::entities::project::Project;
use mcb_domain::ports::repositories::{AgentRepository, MemoryRepository, ProjectRepository};
use mcb_providers::database::{
    create_agent_repository_from_executor, create_memory_repository_with_executor,
    create_project_repository_from_executor,
};

// ============================================================================
// Helper Functions
// ============================================================================

async fn setup_repositories() -> (
    Arc<dyn AgentRepository>,
    Arc<dyn MemoryRepository>,
    Arc<dyn ProjectRepository>,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let (memory_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("Failed to create executor");
    let agent_repo = create_agent_repository_from_executor(Arc::clone(&executor));
    let project_repo = create_project_repository_from_executor(Arc::clone(&executor));
    (agent_repo, memory_repo, project_repo, temp_dir)
}

fn create_test_project(id: &str) -> Project {
    let now = 1000000i64;
    Project {
        id: id.to_string(),
        name: "Test Project".to_string(),
        path: "/tmp/test".to_string(),
        created_at: now,
        updated_at: now,
    }
}

fn create_test_session_summary(id: &str, project_id: &str) -> SessionSummary {
    let now = 1000000i64;
    SessionSummary {
        id: id.to_string(),
        project_id: project_id.to_string(),
        session_id: "linked-session-id".to_string(), // Just a string for now
        topics: vec![],
        decisions: vec![],
        next_steps: vec![],
        key_files: vec![],
        created_at: now,
    }
}

fn create_test_agent_session(id: &str, session_summary_id: &str) -> AgentSession {
    let now = 1000000i64;
    AgentSession {
        id: id.to_string(),
        session_summary_id: session_summary_id.to_string(),
        agent_type: mcb_domain::entities::agent::AgentType::Sisyphus,
        model: "test-model".to_string(),
        parent_session_id: None,
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: Some("Test Prompt".to_string()),
        result_summary: None,
        token_count: None,
        tool_calls_count: None,
        delegations_count: None,
    }
}

fn create_test_tool_call(id: &str, session_id: &str) -> ToolCall {
    let now = 1000000i64;
    ToolCall {
        id: id.to_string(),
        session_id: session_id.to_string(),
        tool_name: "test_tool".to_string(),
        params_summary: Some("{}".to_string()),
        success: true,
        error_message: None,
        duration_ms: Some(100),
        created_at: now,
    }
}

// ============================================================================
// Agent Repository Tests
// ============================================================================

#[tokio::test]
async fn test_create_agent_session() {
    let (agent_repo, memory_repo, project_repo, _temp) = setup_repositories().await;

    // Prerequisite: Create Project
    let project = create_test_project("proj-1");
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // Prerequisite: Create SessionSummary
    let summary = create_test_session_summary("sess-1", "proj-1");
    memory_repo
        .store_session_summary(&summary)
        .await
        .expect("Failed to store session summary");

    // Test: Create AgentSession
    let session = create_test_agent_session("agent-1", "sess-1");
    agent_repo
        .create_session(&session)
        .await
        .expect("Failed to create agent session");

    // Verify
    let retrieved = agent_repo
        .get_session("agent-1")
        .await
        .expect("Failed to get session");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "agent-1");
}

#[tokio::test]
async fn test_store_tool_call() {
    let (agent_repo, memory_repo, project_repo, _temp) = setup_repositories().await;

    // Prerequisite: Create Project
    let project = create_test_project("proj-1");
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // Prerequisite: Create SessionSummary and AgentSession
    let summary = create_test_session_summary("sess-2", "proj-1");
    memory_repo
        .store_session_summary(&summary)
        .await
        .expect("Failed to store session summary");

    let session = create_test_agent_session("agent-2", "sess-2");
    agent_repo
        .create_session(&session)
        .await
        .expect("Failed to create agent session");

    // Test: Store ToolCall
    let tool_call = create_test_tool_call("tool-1", "agent-2");
    agent_repo
        .store_tool_call(&tool_call)
        .await
        .expect("Failed to store tool call");
}
