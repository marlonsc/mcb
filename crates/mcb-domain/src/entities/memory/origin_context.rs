use serde::{Deserialize, Serialize};

/// Contextual information about the origin of an operation or observation.
///
/// Tracks the "who, what, where, and when" of an action within the system.
/// Used for audit trails, debugging, and correlating events across sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
// TODO(architecture): Add id: Uuid or similar identity field to entity.
// Currently relies on specific fields for identification which may not be unique.
// TODO(CA004): Entity OriginContext missing id/uuid field - Add id: Uuid or similar identity field to entity
pub struct OriginContext {
    /// The ID of the organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// The ID of the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// The ID of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Stable hash of session ID for safe correlation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id_hash: Option<String>,
    /// The ID of the parent session when delegated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    /// Stable hash of parent session ID for safe correlation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id_hash: Option<String>,
    /// The ID of the execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    /// The name of the tool being executed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// The ID of the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<String>,
    /// The path to the repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<String>,
    /// Operator/user identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    /// Machine/host fingerprint identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    /// Agent program or IDE identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_program: Option<String>,
    /// Model identifier used for execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    /// Whether this execution was delegated to a subagent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegated: Option<bool>,
    /// The ID of the worktree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_id: Option<String>,
    /// The path to the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// The current git branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// The current git commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Timestamp of the observation/action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}
