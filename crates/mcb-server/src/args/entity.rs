use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

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

tool_enum! {
/// CRUD actions for plan-related entity resources.
pub enum PlanEntityAction {
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
}
}

tool_enum! {
/// Plan-related resource types for entity operations.
pub enum PlanEntityResource {
    /// Plan resource.
    Plan,
    /// Plan version resource.
    Version,
    /// Plan review resource.
    Review,
}
}

tool_schema! {
/// Arguments for plan-related entity operations.
pub struct PlanEntityArgs {
    /// Action: create, get, update, list, delete
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: PlanEntityAction,

    /// Resource: plan, version, review
    #[schemars(description = "Resource: plan, version, review")]
    pub resource: PlanEntityResource,

    /// Resource ID (for get/update/delete)
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    /// Organization ID (uses default if omitted)
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Project ID (for plan listing)
    #[schemars(description = "Project ID (for plan listing)")]
    pub project_id: Option<String>,

    /// Plan ID (for version listing)
    #[schemars(description = "Plan ID (for version listing)")]
    pub plan_id: Option<String>,

    /// Plan version ID (for review listing)
    #[schemars(description = "Plan version ID (for review listing)")]
    pub plan_version_id: Option<String>,

    /// Data payload for create/update (JSON object)
    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}
}

tool_enum! {
/// CRUD actions for organization-related entity resources.
pub enum OrgEntityAction {
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
}
}

tool_enum! {
/// Organization-related resource types for entity operations.
pub enum OrgEntityResource {
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
/// Arguments for organization-related entity operations.
pub struct OrgEntityArgs {
    /// Action: create, get, update, list, delete
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: OrgEntityAction,
    /// Resource: org, user, team, team_member, api_key
    #[schemars(description = "Resource: org, user, team, team_member, api_key")]
    pub resource: OrgEntityResource,
    /// Resource ID (for get/update/delete)
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,
    /// Organization ID (for listing users/teams/api_keys)
    #[schemars(description = "Organization ID (for listing users/teams/api_keys)")]
    pub org_id: Option<String>,
    /// Team ID (for listing members)
    #[schemars(description = "Team ID (for listing members)")]
    pub team_id: Option<String>,
    /// User ID (for removing team member)
    #[schemars(description = "User ID (for removing team member)")]
    pub user_id: Option<String>,
    /// Email (for user lookup by email)
    #[schemars(description = "Email (for user lookup by email)")]
    pub email: Option<String>,
    /// Data payload for create/update (JSON object)
    #[schemars(description = "Data payload for create/update (JSON object)")]
    #[schemars(with = "serde_json::Value")]
    pub data: Option<serde_json::Value>,
}
}

tool_enum! {
/// CRUD actions for issue-related entity resources.
pub enum IssueEntityAction {
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
}
}

tool_enum! {
/// Issue-related resource types for entity operations.
pub enum IssueEntityResource {
    /// Issue resource.
    Issue,
    /// Issue comment resource.
    Comment,
    /// Issue label resource.
    Label,
    /// Issue label assignment resource.
    LabelAssignment,
}
}

tool_schema! {
/// Arguments for issue-related entity operations.
pub struct IssueEntityArgs {
    /// Action: create, get, update, list, delete
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: IssueEntityAction,

    /// Resource: issue, comment, label, label_assignment
    #[schemars(description = "Resource: issue, comment, label, label_assignment")]
    pub resource: IssueEntityResource,

    /// Resource ID (for get/update/delete)
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    /// Organization ID (uses default if omitted)
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Project ID (for issue/label listing)
    #[schemars(description = "Project ID (for issue/label listing)")]
    pub project_id: Option<String>,

    /// Issue ID (for comment listing and label assignments)
    #[schemars(description = "Issue ID (for comment listing and label assignments)")]
    pub issue_id: Option<String>,

    /// Label ID (for label unassignment)
    #[schemars(description = "Label ID (for label unassignment)")]
    pub label_id: Option<String>,

    /// Data payload for create/update (JSON object)
    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}
}
