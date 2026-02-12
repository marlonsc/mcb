use serde::{Deserialize, Serialize};

/// Contextual information about the origin of an operation or observation.
#[allow(missing_docs)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
