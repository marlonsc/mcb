use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::args::macros::{tool_enum, tool_schema};

tool_enum! {
/// CRUD actions for VCS entity resources.
pub enum VcsEntityAction {
    /// Create a new entity.
    Create,
    /// Get an entity by ID.
    Get,
    /// Update an existing entity.
    Update,
    /// List entities matching criteria.
    List,
    /// Delete an entity by ID.
    Delete,
    /// Release an assignment (VCS assignment only).
    Release,
}
}

tool_enum! {
/// VCS resource types for entity operations.
pub enum VcsEntityResource {
    /// VCS repository resource.
    Repository,
    /// VCS branch resource.
    Branch,
    /// VCS worktree resource.
    Worktree,
    /// VCS assignment resource.
    Assignment,
}
}

tool_schema! {
/// Arguments for VCS entity operations.
pub struct VcsEntityArgs {
    /// Action: create, get, update, list, delete, release
    #[schemars(description = "Action: create, get, update, list, delete, release")]
    pub action: VcsEntityAction,

    /// Resource: repository, branch, worktree, assignment
    #[schemars(description = "Resource: repository, branch, worktree, assignment")]
    pub resource: VcsEntityResource,

    /// Resource ID (for get/update/delete/release)
    #[schemars(description = "Resource ID (for get/update/delete/release)")]
    pub id: Option<String>,

    /// Organization ID (uses default if omitted)
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Project ID (for repository listing)
    #[schemars(description = "Project ID (for repository listing)")]
    pub project_id: Option<String>,

    /// Repository ID (for branch/worktree listing)
    #[schemars(description = "Repository ID (for branch/worktree listing)")]
    pub repository_id: Option<String>,

    /// Worktree ID (for assignment listing)
    #[schemars(description = "Worktree ID (for assignment listing)")]
    pub worktree_id: Option<String>,

    /// Data payload for create/update. phase: {name, status, order}; issue: {title, description?, status?, priority?}; dependency: {from_issue_id, to_issue_id, kind?}; decision: {title, rationale, impact?, status?}
    #[schemars(
        description = "Data payload for create/update. phase: {name, status, order}; issue: {title, description?, status?, priority?}; dependency: {from_issue_id, to_issue_id, kind?}; decision: {title, rationale, impact?, status?}",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}
}
