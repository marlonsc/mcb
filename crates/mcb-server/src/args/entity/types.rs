//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::args::schema_helpers::ObjectDataSchema;

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
    #[schemars(description = "Action: create, get, update, list, delete, release")]
    pub action: EntityAction,
    /// Target resource type for consolidated entity operations.
    #[schemars(description = "Resource type")]
    pub resource: EntityResource,
    /// JSON payload for create/update actions.
    #[schemars(description = "Data payload for create/update (JSON object)", with = "ObjectDataSchema")]
    pub data: Option<serde_json::Value>,
    /// Resource ID (for get/update/delete/release).
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    // --- context (auto-injected, hidden from MCP schema) ---
    /// Organization ID (auto-injected).
    #[schemars(skip)]
    pub org_id: Option<String>,
    /// Project ID (auto-injected).
    #[schemars(skip)]
    pub project_id: Option<String>,
    /// Repository ID (auto-injected).
    #[schemars(skip)]
    pub repository_id: Option<String>,
    /// Worktree ID (auto-injected).
    #[schemars(skip)]
    pub worktree_id: Option<String>,
    /// Plan ID (auto-injected).
    #[schemars(skip)]
    pub plan_id: Option<String>,
    /// Plan version ID (auto-injected).
    #[schemars(skip)]
    pub plan_version_id: Option<String>,
    /// Issue ID (auto-injected).
    #[schemars(skip)]
    pub issue_id: Option<String>,
    /// Label ID (auto-injected).
    #[schemars(skip)]
    pub label_id: Option<String>,
    /// Team ID (auto-injected).
    #[schemars(skip)]
    pub team_id: Option<String>,
    /// User ID (auto-injected).
    #[schemars(skip)]
    pub user_id: Option<String>,
    /// User email (auto-injected).
    #[schemars(skip)]
    pub email: Option<String>,
}
}
