//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Custom HTTP header name constants — Single Source of Truth
//!
//! All `X-*` provenance / execution-context headers used across transports.
//! Server, client, and test code MUST import these instead of inline strings.

define_str_consts! {
    /// HTTP header: workspace root path.
    HEADER_WORKSPACE_ROOT = "X-Workspace-Root";
    /// HTTP header: repository path.
    HEADER_REPO_PATH = "X-Repo-Path";
    /// HTTP header: repository identifier.
    HEADER_REPO_ID = "X-Repo-Id";
    /// HTTP header: session identifier.
    HEADER_SESSION_ID = "X-Session-Id";
    /// HTTP header: parent session identifier.
    HEADER_PARENT_SESSION_ID = "X-Parent-Session-Id";
    /// HTTP header: project identifier.
    HEADER_PROJECT_ID = "X-Project-Id";
    /// HTTP header: worktree identifier.
    HEADER_WORKTREE_ID = "X-Worktree-Id";
    /// HTTP header: operator / user identifier.
    HEADER_OPERATOR_ID = "X-Operator-Id";
    /// HTTP header: machine identifier.
    HEADER_MACHINE_ID = "X-Machine-Id";
    /// HTTP header: agent program (IDE) identifier.
    HEADER_AGENT_PROGRAM = "X-Agent-Program";
    /// HTTP header: model identifier.
    HEADER_MODEL_ID = "X-Model-Id";
    /// HTTP header: delegation flag.
    HEADER_DELEGATED = "X-Delegated";
    /// HTTP header: org identifier.
    HEADER_ORG_ID = "X-Org-Id";
}

/// All provenance header-to-context-key mappings.
///
/// Used by `build_overrides()` to extract provenance headers from HTTP
/// requests and map them to execution context keys.
pub const PROVENANCE_HEADER_MAPPINGS: &[(&str, &str)] = &[
    (HEADER_WORKSPACE_ROOT, "workspace_root"),
    (HEADER_REPO_PATH, "repo_path"),
    (HEADER_REPO_ID, "repo_id"),
    (HEADER_SESSION_ID, "session_id"),
    (HEADER_PARENT_SESSION_ID, "parent_session_id"),
    (HEADER_PROJECT_ID, "project_id"),
    (HEADER_WORKTREE_ID, "worktree_id"),
    (HEADER_OPERATOR_ID, "operator_id"),
    (HEADER_MACHINE_ID, "machine_id"),
    (HEADER_AGENT_PROGRAM, "agent_program"),
    (HEADER_MODEL_ID, "model_id"),
    (HEADER_DELEGATED, "delegated"),
];
