//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_crud_action_enum! {
/// CRUD actions for VCS entity resources.
pub enum VcsEntityAction {
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

entity_args_schema! {
/// Arguments for VCS entity operations.
pub struct VcsEntityArgs {
    action: VcsEntityAction,
    action_desc: "Action: create, get, update, list, delete, release",
    resource: VcsEntityResource,
    resource_desc: "Resource: repository, branch, worktree, assignment",
    /// Project ID (for repository listing)
    project_id: Option<String> => "Project ID (for repository listing)",
    /// Repository ID (for branch/worktree listing)
    repository_id: Option<String> => "Repository ID (for branch/worktree listing)",
    /// Worktree ID (for assignment listing)
    worktree_id: Option<String> => "Worktree ID (for assignment listing)",
}
}
