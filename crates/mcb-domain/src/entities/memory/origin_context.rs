use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;

/// Contextual information about the origin of an operation or observation.
///
/// Tracks the "who, what, where, and when" of an action within the system.
/// Used for audit trails, debugging, and correlating events across sessions.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
// TODO(architecture): Add id: Uuid or similar identity field to entity.
// Currently relies on specific fields for identification which may not be unique.
// TODO(CA004): Entity OriginContext missing id/uuid field - Add id: Uuid or similar identity field to entity
pub struct OriginContext {
    /// The ID of the organization.
    pub org_id: Option<String>,
    /// The ID of the project.
    pub project_id: Option<String>,
    /// The ID of the session.
    pub session_id: Option<String>,
    /// Stable hash of session ID for safe correlation.
    pub session_id_hash: Option<String>,
    /// The ID of the parent session when delegated.
    pub parent_session_id: Option<String>,
    /// Stable hash of parent session ID for safe correlation.
    pub parent_session_id_hash: Option<String>,
    /// The ID of the execution.
    pub execution_id: Option<String>,
    /// The name of the tool being executed.
    pub tool_name: Option<String>,
    /// The ID of the repository.
    pub repo_id: Option<String>,
    /// The path to the repository.
    pub repo_path: Option<String>,
    /// Operator/user identifier.
    pub operator_id: Option<String>,
    /// Machine/host fingerprint identifier.
    pub machine_id: Option<String>,
    /// Agent program or IDE identifier.
    pub agent_program: Option<String>,
    /// Model identifier used for execution.
    pub model_id: Option<String>,
    /// Whether this execution was delegated to a subagent.
    pub delegated: Option<bool>,
    /// The ID of the worktree.
    pub worktree_id: Option<String>,
    /// The path to the file.
    pub file_path: Option<String>,
    /// The current git branch.
    pub branch: Option<String>,
    /// The current git commit.
    pub commit: Option<String>,
    /// Timestamp of the observation/action.
    pub timestamp: Option<i64>,
}
