use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

mod issue;
mod org;
mod plan;
mod vcs;

pub use issue::*;
pub use org::*;
pub use plan::*;
pub use vcs::*;

tool_enum! {
/// CRUD actions available for entity resources.
pub enum EntityAction {
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
/// Target resource types for consolidated entity operations.
pub enum EntityResource {
    /// VCS repository resource.
    Repository,
    /// VCS branch resource.
    Branch,
    /// VCS worktree resource.
    Worktree,
    /// VCS assignment resource.
    Assignment,
    /// Plan resource.
    Plan,
    /// Plan version resource.
    Version,
    /// Plan review resource.
    Review,
    /// Issue resource.
    Issue,
    /// Issue comment resource.
    Comment,
    /// Issue label resource.
    Label,
    /// Issue label assignment resource.
    LabelAssignment,
    /// Organization resource.
    Org,
    /// User resource.
    User,
    /// Team resource.
    Team,
    /// Team member resource.
    TeamMember,
    /// API key resource.
    ApiKey,
}
}

tool_schema! {
/// Arguments for the consolidated `entity` MCP tool.
pub struct EntityArgs {
    /// CRUD actions for entity resources.
    pub action: EntityAction,
    /// Target resource type for consolidated entity operations.
    pub resource: EntityResource,
    /// JSON payload for create/update actions.
    #[schemars(with = "serde_json::Value")]
    pub data: Option<serde_json::Value>,
    /// Resource ID (for get/update/delete/release).
    pub id: Option<String>,
    /// Organization ID.
    pub org_id: Option<String>,
    /// Project ID (project-scoped list operations).
    pub project_id: Option<String>,
    /// Repository ID (branch/worktree list operations).
    pub repository_id: Option<String>,
    /// Worktree ID (assignment list operations).
    pub worktree_id: Option<String>,
    /// Plan ID (version list operations).
    pub plan_id: Option<String>,
    /// Plan version ID (review list operations).
    pub plan_version_id: Option<String>,
    /// Issue ID (comment/list/label assignment operations).
    pub issue_id: Option<String>,
    /// Label ID (label unassignment operations).
    pub label_id: Option<String>,
    /// Team ID (team member list operations).
    pub team_id: Option<String>,
    /// User ID (team member delete operations).
    pub user_id: Option<String>,
    /// User email (lookup operations).
    pub email: Option<String>,
}
}
