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

// ---------------------------------------------------------------------------
// Centralized workspace root
// ---------------------------------------------------------------------------

/// Returns the workspace root directory by traversing up from the crate manifest dir.
///
/// Each crate sits at `<workspace>/crates/<crate>`, so 2 ancestors up is the workspace root.
/// Returns an error instead of panicking for composability with `TestResult`.
///
/// # Example
/// ```rust,ignore
/// use mcb_domain::test_utils::workspace_root;
///
/// let root = workspace_root()?;
/// assert!(root.join("Cargo.toml").exists());
/// ```
///
/// # Errors
///
/// Returns an error if `CARGO_MANIFEST_DIR` has fewer than 2 parent directories.
pub fn workspace_root() -> TestResult<std::path::PathBuf> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .ok_or_else(|| {
            "workspace root not found (CARGO_MANIFEST_DIR has fewer than 2 parents)".into()
        })
        .map(std::path::Path::to_path_buf)
}

// ---------------------------------------------------------------------------
// Common test identity constants
// ---------------------------------------------------------------------------

/// Default test organization ID — use across all crates for consistency.
pub const TEST_ORG_ID: &str = "test-org";

/// Default test project ID.
pub const TEST_PROJECT_ID: &str = "test-project";

/// Default test session ID.
pub const TEST_SESSION_ID: &str = "test-session";

/// Default test user email.
pub const TEST_USER_EMAIL: &str = "test@example.com";

/// Default test timestamp (`2023-11-14T22:13:20Z`).
pub const TEST_TIMESTAMP: i64 = 1_700_000_000;

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
        org_id: TEST_ORG_ID.to_owned(),
        name: TEST_PROJECT_ID.to_owned(),
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
        org_id: TEST_ORG_ID.to_owned(),
        email: TEST_USER_EMAIL.to_owned(),
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
        org_id: TEST_ORG_ID.to_owned(),
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
        started_at: TEST_TIMESTAMP,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: Some("Test Prompt".to_owned()),
        result_summary: None,
        token_count: Some(0),
        tool_calls_count: Some(0),
        delegations_count: Some(0),
        project_id: Some(TEST_PROJECT_ID.to_owned()),
        worktree_id: None,
    }
}

/// Creates a test `ToolCall` with default values.
#[must_use]
pub fn create_test_tool_call(id: &str) -> ToolCall {
    ToolCall {
        id: id.to_owned(),
        session_id: TEST_SESSION_ID.to_owned(),
        tool_name: "test_tool".to_owned(),
        params_summary: Some("check=true".to_owned()),
        success: true,
        error_message: None,
        duration_ms: Some(100),
        created_at: TEST_TIMESTAMP,
    }
}

/// Creates a test `Checkpoint` with default values.
#[must_use]
pub fn create_test_checkpoint(id: &str) -> Checkpoint {
    Checkpoint {
        id: id.to_owned(),
        session_id: TEST_SESSION_ID.to_owned(),
        checkpoint_type: CheckpointType::Git,
        description: "Test Checkpoint".to_owned(),
        snapshot_data: serde_json::json!({"status": "clean"}),
        created_at: TEST_TIMESTAMP,
        restored_at: None,
        expired: false,
    }
}
