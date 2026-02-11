//! Unit tests for SqliteAgentRepository
//!
//! Tests cover creation and storage of agent sessions and tool calls, verifying
//! SQL schema integration and foreign key constraints.

use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, ToolCall};
use mcb_domain::entities::memory::SessionSummary;
use mcb_domain::entities::project::Project;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::{AgentRepository, MemoryRepository, ProjectRepository};
use mcb_providers::database::{
    create_agent_repository_from_executor, create_memory_repository_with_executor,
    create_project_repository_from_executor,
};

async fn setup_repositories() -> (
    Arc<dyn AgentRepository>,
    Arc<dyn MemoryRepository>,
    Arc<dyn ProjectRepository>,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let (memory_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("Failed to create executor");
    seed_default_org(executor.as_ref()).await;
    let agent_repo = create_agent_repository_from_executor(Arc::clone(&executor));
    let project_repo = create_project_repository_from_executor(Arc::clone(&executor));
    (agent_repo, memory_repo, project_repo, executor, temp_dir)
}

async fn seed_default_org(executor: &dyn DatabaseExecutor) {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("{}".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed default org");
}

fn create_test_project(id: &str) -> Project {
    let now = 1000000i64;
    Project {
        id: id.to_string(),
        org_id: mcb_domain::constants::keys::DEFAULT_ORG_ID.to_string(),
        name: format!("Test Project {id}"),
        path: format!("/tmp/test/{id}"),
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
        project_id: None,
        worktree_id: None,
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

async fn seed_worktree(
    executor: &dyn DatabaseExecutor,
    project_id: &str,
    repository_id: &str,
    branch_id: &str,
    worktree_id: &str,
) {
    let now = 1_000_000_i64;

    executor
        .execute(
            "INSERT INTO repositories (id, org_id, project_id, name, url, local_path, vcs_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(repository_id.to_string()),
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String(project_id.to_string()),
                SqlParam::String(format!("repo-{repository_id}")),
                SqlParam::String("https://example.com/repo.git".to_string()),
                SqlParam::String(format!("/tmp/{repository_id}")),
                SqlParam::String("git".to_string()),
                SqlParam::I64(now),
                SqlParam::I64(now),
            ],
        )
        .await
        .expect("seed repository");

    executor
        .execute(
            "INSERT INTO branches (id, repository_id, name, is_default, head_commit, upstream, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(branch_id.to_string()),
                SqlParam::String(repository_id.to_string()),
                SqlParam::String("main".to_string()),
                SqlParam::Bool(true),
                SqlParam::String("abc123".to_string()),
                SqlParam::Null,
                SqlParam::I64(now),
            ],
        )
        .await
        .expect("seed branch");

    executor
        .execute(
            "INSERT INTO worktrees (id, repository_id, branch_id, path, status, assigned_agent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(worktree_id.to_string()),
                SqlParam::String(repository_id.to_string()),
                SqlParam::String(branch_id.to_string()),
                SqlParam::String(format!("/tmp/worktree-{worktree_id}")),
                SqlParam::String("active".to_string()),
                SqlParam::Null,
                SqlParam::I64(now),
                SqlParam::I64(now),
            ],
        )
        .await
        .expect("seed worktree");
}

// ============================================================================
// Agent Repository Tests
// ============================================================================

#[tokio::test]
async fn test_create_agent_session() {
    let (agent_repo, memory_repo, project_repo, _executor, _temp) = setup_repositories().await;

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
    let (agent_repo, memory_repo, project_repo, _executor, _temp) = setup_repositories().await;

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

#[tokio::test]
async fn test_list_sessions_by_project() {
    let (agent_repo, memory_repo, project_repo, _executor, _temp) = setup_repositories().await;

    let project_1 = create_test_project("proj-1");
    let project_2 = create_test_project("proj-2");
    project_repo
        .create(&project_1)
        .await
        .expect("create project 1");
    project_repo
        .create(&project_2)
        .await
        .expect("create project 2");

    let summary_1 = create_test_session_summary("sess-1", "proj-1");
    let summary_2 = create_test_session_summary("sess-2", "proj-2");
    memory_repo
        .store_session_summary(&summary_1)
        .await
        .expect("store summary 1");
    memory_repo
        .store_session_summary(&summary_2)
        .await
        .expect("store summary 2");

    let mut session_1 = create_test_agent_session("agent-1", "sess-1");
    session_1.project_id = Some("proj-1".to_string());
    let mut session_2 = create_test_agent_session("agent-2", "sess-2");
    session_2.project_id = Some("proj-2".to_string());

    agent_repo
        .create_session(&session_1)
        .await
        .expect("create session 1");
    agent_repo
        .create_session(&session_2)
        .await
        .expect("create session 2");

    let sessions = agent_repo
        .list_sessions_by_project("proj-1")
        .await
        .expect("list sessions by project");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "agent-1");
    assert_eq!(sessions[0].project_id.as_deref(), Some("proj-1"));
}

#[tokio::test]
async fn test_list_sessions_by_worktree() {
    let (agent_repo, memory_repo, project_repo, executor, _temp) = setup_repositories().await;

    let project_1 = create_test_project("proj-1");
    project_repo
        .create(&project_1)
        .await
        .expect("create project");

    seed_worktree(executor.as_ref(), "proj-1", "repo-1", "branch-1", "wt-1").await;
    seed_worktree(executor.as_ref(), "proj-1", "repo-2", "branch-2", "wt-2").await;

    let summary_1 = create_test_session_summary("sess-1", "proj-1");
    let summary_2 = create_test_session_summary("sess-2", "proj-1");
    memory_repo
        .store_session_summary(&summary_1)
        .await
        .expect("store summary 1");
    memory_repo
        .store_session_summary(&summary_2)
        .await
        .expect("store summary 2");

    let mut session_1 = create_test_agent_session("agent-1", "sess-1");
    session_1.worktree_id = Some("wt-1".to_string());
    let mut session_2 = create_test_agent_session("agent-2", "sess-2");
    session_2.worktree_id = Some("wt-2".to_string());

    agent_repo
        .create_session(&session_1)
        .await
        .expect("create session 1");
    agent_repo
        .create_session(&session_2)
        .await
        .expect("create session 2");

    let sessions = agent_repo
        .list_sessions_by_worktree("wt-1")
        .await
        .expect("list sessions by worktree");

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "agent-1");
    assert_eq!(sessions[0].worktree_id.as_deref(), Some("wt-1"));
}
