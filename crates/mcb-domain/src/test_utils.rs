//!
//! **Documentation**: [docs/modules/domain.md#testing-utilities](../../../docs/modules/domain.md#testing-utilities)
//!
//! Centralized test utilities for the entire workspace.
//! All crates MUST import shared test types from here instead of defining them locally.

/// Centralized test result type for all test functions across the workspace.
///
/// Use this instead of defining per-file `type TestResult` aliases.
///
/// # Example
/// ```rust,ignore
/// use mcb_domain::test_utils::TestResult;
///
/// #[test]
/// fn my_test() -> TestResult {
///     let value = some_fallible_fn()?;
///     assert_eq!(value, 42);
///     Ok(())
/// }
/// ```
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

use crate::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, ToolCall,
};
use crate::entities::project::{
    IssueStatus, IssueType, PhaseStatus, Project, ProjectIssue, ProjectPhase,
};
use crate::entities::user::User;
use uuid::Uuid;

/// Creates a test `Project` with default values.
#[must_use]
pub fn create_test_project(id: &str) -> Project {
    Project {
        id: id.to_owned(),
        org_id: "test-org".to_owned(),
        name: "test-project".to_owned(),
        path: "/tmp/test-project".to_owned(),
        created_at: 0,
        updated_at: 0,
    }
}

/// Creates a test `User` with default values.
#[must_use]
pub fn create_test_user() -> User {
    User {
        id: Uuid::new_v4().to_string(),
        org_id: "test-org".to_owned(),
        email: "test@example.com".to_owned(),
        display_name: "Test User".to_owned(),
        role: crate::entities::user::UserRole::Member,
        api_key_hash: None,
        created_at: 0,
        updated_at: 0,
    }
}

/// Creates a test `ProjectPhase` with default values.
#[must_use]
pub fn create_test_phase(id: &str, project_id: &str) -> ProjectPhase {
    ProjectPhase {
        id: id.to_owned(),
        project_id: project_id.to_owned(),
        name: format!("Phase {id}"),
        description: "Test Phase Description".to_owned(),
        sequence: 1,
        status: PhaseStatus::Planned,
        started_at: None,
        completed_at: None,
        created_at: 1000,
        updated_at: 1000,
    }
}

/// Creates a test `ProjectIssue` with default values.
#[must_use]
pub fn create_test_issue(id: &str, project_id: &str) -> ProjectIssue {
    ProjectIssue {
        id: id.to_owned(),
        org_id: "test-org".to_owned(),
        project_id: project_id.to_owned(),
        created_by: "test-user".to_owned(),
        phase_id: None,
        title: format!("Issue {id}"),
        description: "Test Issue Description".to_owned(),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec![],
        estimated_minutes: None,
        actual_minutes: None,
        notes: String::new(),
        design: String::new(),
        parent_issue_id: None,
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
        closed_reason: String::new(),
    }
}

/// Creates a test `AgentSession` with default values.
#[must_use]
pub fn create_test_agent_session(id: &str) -> AgentSession {
    AgentSession {
        id: id.to_owned(),
        session_summary_id: Uuid::new_v4().to_string(),
        agent_type: AgentType::Sisyphus,
        model: "claude-sonnet".to_owned(),
        parent_session_id: None,
        started_at: 1700000000,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: Some("Test Prompt".to_owned()),
        result_summary: None,
        token_count: Some(0),
        tool_calls_count: Some(0),
        delegations_count: Some(0),
        project_id: Some("test-project".to_owned()),
        worktree_id: None,
    }
}

/// Creates a test `ToolCall` with default values.
#[must_use]
pub fn create_test_tool_call(id: &str) -> ToolCall {
    ToolCall {
        id: id.to_owned(),
        session_id: "test-session".to_owned(),
        tool_name: "test_tool".to_owned(),
        params_summary: Some("check=true".to_owned()),
        success: true,
        error_message: None,
        duration_ms: Some(100),
        created_at: 1700000000,
    }
}

/// Creates a test `Checkpoint` with default values.
#[must_use]
pub fn create_test_checkpoint(id: &str) -> Checkpoint {
    Checkpoint {
        id: id.to_owned(),
        session_id: "test-session".to_owned(),
        checkpoint_type: CheckpointType::Git,
        description: "Test Checkpoint".to_owned(),
        snapshot_data: serde_json::json!({"status": "clean"}),
        created_at: 1700000000,
        restored_at: None,
        expired: false,
    }
}
