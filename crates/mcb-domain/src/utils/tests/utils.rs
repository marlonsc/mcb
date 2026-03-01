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
use tempfile::TempDir;
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

// ---------------------------------------------------------------------------
// Extended test constants (migrated from mcb-server/tests/utils/test_fixtures)
// ---------------------------------------------------------------------------

/// Default test repository name.
pub const TEST_REPO_NAME: &str = "test-repo";

/// Default embedding dimensions (`FastEmbed` BGE-small-en-v1.5).
pub const TEST_EMBEDDING_DIMENSIONS: usize = 384;

/// Organization A identifier for multi-tenant tests.
pub const TEST_ORG_ID_A: &str = "test-org-a";

/// Organization B identifier for multi-tenant tests.
pub const TEST_ORG_ID_B: &str = "test-org-b";

/// Default golden-test collection name.
pub const GOLDEN_COLLECTION: &str = "mcb_golden_test";

/// Expected files in `sample_codebase` for search assertions.
pub const SAMPLE_CODEBASE_FILES: &[&str] = &[
    "embedding.rs",
    "vector_store.rs",
    "handlers.rs",
    "cache.rs",
    "di.rs",
    "error.rs",
    "chunking.rs",
];

// ---------------------------------------------------------------------------
// Entity fixture builders (migrated from mcb-server/tests/utils/test_fixtures)
// ---------------------------------------------------------------------------

use crate::entities::api_key::ApiKey;
use crate::entities::organization::Organization;
use crate::entities::team::{Team, TeamMember, TeamMemberRole};
use crate::value_objects::ids::TeamMemberId;

/// Creates a test `Organization` with default values.
#[must_use]
pub fn create_test_organization(id: &str) -> Organization {
    Organization {
        id: id.to_owned(),
        name: format!("Test Org {id}"),
        slug: format!("test-org-{id}"),
        settings_json: "{}".to_owned(),
        created_at: TEST_TIMESTAMP,
        updated_at: TEST_TIMESTAMP,
    }
}

/// Creates a test `User` with Admin role.
#[must_use]
pub fn create_test_admin_user(org_id: &str, email: &str) -> User {
    User {
        id: Uuid::new_v4().to_string(),
        org_id: org_id.to_owned(),
        email: email.to_owned(),
        display_name: email.split('@').next().unwrap_or("Test Admin").to_owned(),
        role: crate::entities::user::UserRole::Admin,
        api_key_hash: None,
        created_at: TEST_TIMESTAMP,
        updated_at: TEST_TIMESTAMP,
    }
}

/// Creates a test `User` with custom org/email (Member role).
#[must_use]
pub fn create_test_user_with(org_id: &str, email: &str) -> User {
    User {
        id: Uuid::new_v4().to_string(),
        org_id: org_id.to_owned(),
        email: email.to_owned(),
        display_name: email.split('@').next().unwrap_or("Test User").to_owned(),
        role: crate::entities::user::UserRole::Member,
        api_key_hash: None,
        created_at: TEST_TIMESTAMP,
        updated_at: TEST_TIMESTAMP,
    }
}

/// Creates a test `Team` with default values.
#[must_use]
pub fn create_test_team(org_id: &str, name: &str) -> Team {
    Team {
        id: Uuid::new_v4().to_string(),
        org_id: org_id.to_owned(),
        name: name.to_owned(),
        created_at: TEST_TIMESTAMP,
    }
}

/// Creates a test `TeamMember` with default values.
#[must_use]
pub fn create_test_team_member(team_id: &str, user_id: &str) -> TeamMember {
    TeamMember {
        id: TeamMemberId::from_string(&format!("{team_id}:{user_id}")),
        team_id: team_id.to_owned(),
        user_id: user_id.to_owned(),
        role: TeamMemberRole::Member,
        joined_at: TEST_TIMESTAMP,
    }
}

/// Creates a test `ApiKey` with default values.
#[must_use]
pub fn create_test_api_key(user_id: &str, org_id: &str, name: &str) -> ApiKey {
    ApiKey {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_owned(),
        org_id: org_id.to_owned(),
        name: name.to_owned(),
        key_hash: format!("hash_{}", Uuid::new_v4()),
        scopes_json: "[\"read\", \"write\"]".to_owned(),
        expires_at: None,
        revoked_at: None,
        created_at: TEST_TIMESTAMP,
    }
}

// ---------------------------------------------------------------------------
// Workspace / codebase helpers (migrated from mcb-server/tests/utils/test_fixtures)
// ---------------------------------------------------------------------------

/// Create a temporary codebase directory with sample code files.
///
/// Returns `(TempDir, PathBuf)` — keep `TempDir` alive for the test.
#[must_use]
pub fn create_temp_codebase() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let codebase_path = temp_dir.path().to_path_buf();

    std::fs::write(
        codebase_path.join("lib.rs"),
        "//! Sample library\npub fn hello() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .expect("Failed to write lib.rs");

    std::fs::write(
        codebase_path.join("main.rs"),
        "fn main() {\n    mylib::hello();\n}\n",
    )
    .expect("Failed to write main.rs");

    let src_dir = codebase_path.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    std::fs::write(
        src_dir.join("utils.rs"),
        "pub fn helper() -> String {\n    \"helper\".to_string()\n}\n",
    )
    .expect("Failed to write utils.rs");

    (temp_dir, codebase_path)
}

/// Create a test indexing result with the given counts.
#[must_use]
pub fn create_test_indexing_result(
    files_processed: usize,
    chunks_created: usize,
    error_count: usize,
) -> crate::ports::IndexingResult {
    let errors = (0..error_count)
        .map(|i| format!("Test error {i}"))
        .collect();

    crate::ports::IndexingResult {
        files_processed,
        chunks_created,
        files_skipped: 0,
        errors,
        operation_id: None,
        status: "completed".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// External service availability detection
// ---------------------------------------------------------------------------

/// Skip a test early (with `Ok(())`) when the named external service is not
/// configured in `config/tests.toml` under `[test_services]`.
///
/// Usage at the top of any `-> TestResult` test function:
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_foo() -> TestResult {
///     require_service!("milvus");
///     // ... rest of the test
/// }
/// ```
#[macro_export]
macro_rules! require_service {
    ($service:expr) => {
        if $crate::utils::tests::services_config::test_service_url($service).is_none() {
            eprintln!("⏭ Skipping: {} not available", $service);
            return Ok(());
        }
    };
}
