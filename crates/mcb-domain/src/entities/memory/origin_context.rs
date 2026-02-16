use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;

/// Contextual information about the origin of an operation or observation.
///
/// Tracks the "who, what, where, and when" of an action within the system.
/// Used for audit trails, debugging, and correlating events across sessions.
///
/// Each `OriginContext` carries a unique `id` (UUID v4) to ensure that
/// distinct origin records can be correlated even when other fields overlap.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct OriginContext {
    /// Unique identity for this origin context (UUID v4, auto-generated on creation).
    #[builder(default_code = "crate::utils::id::generate().to_string()")]
    #[serde(default = "crate::entities::memory::origin_context::generate_id")]
    pub id: String,
    /// The ID of the organization.
    pub org_id: Option<String>,
    /// The ID of the project.
    pub project_id: Option<String>,
    /// The ID of the session.
    pub session_id: Option<String>,
    /// Deterministic UUID v5 correlation of session ID for safe cross-session linking.
    #[serde(alias = "session_id_hash")]
    pub session_id_correlation: Option<String>,
    /// The ID of the parent session when delegated.
    pub parent_session_id: Option<String>,
    /// Deterministic UUID v5 correlation of parent session ID for safe cross-session linking.
    #[serde(alias = "parent_session_id_hash")]
    pub parent_session_id_correlation: Option<String>,
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

pub(crate) fn generate_id() -> String {
    crate::utils::id::generate().to_string()
}

impl Default for OriginContext {
    fn default() -> Self {
        Self {
            id: generate_id(),
            org_id: None,
            project_id: None,
            session_id: None,
            session_id_correlation: None,
            parent_session_id: None,
            parent_session_id_correlation: None,
            execution_id: None,
            tool_name: None,
            repo_id: None,
            repo_path: None,
            operator_id: None,
            machine_id: None,
            agent_program: None,
            model_id: None,
            delegated: None,
            worktree_id: None,
            file_path: None,
            branch: None,
            commit: None,
            timestamp: None,
        }
    }
}
