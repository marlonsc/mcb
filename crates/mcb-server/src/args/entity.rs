use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

tool_enum! {
pub enum EntityAction {
    Create,
    Get,
    Update,
    List,
    Delete,
    Release,
}
}

tool_enum! {
pub enum EntityResource {
    Repository,
    Branch,
    Worktree,
    Assignment,
    Plan,
    Version,
    Review,
    Issue,
    Comment,
    Label,
    LabelAssignment,
    Org,
    User,
    Team,
    TeamMember,
    ApiKey,
}
}

tool_schema! {
pub struct EntityArgs {
    pub action: EntityAction,
    pub resource: EntityResource,
    #[schemars(with = "serde_json::Value")]
    pub data: Option<serde_json::Value>,
    pub id: Option<String>,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
    pub repository_id: Option<String>,
    pub worktree_id: Option<String>,
    pub plan_id: Option<String>,
    pub plan_version_id: Option<String>,
    pub issue_id: Option<String>,
    pub label_id: Option<String>,
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub email: Option<String>,
}
}

tool_enum! {
pub enum VcsEntityAction {
    Create,
    Get,
    Update,
    List,
    Delete,
    Release,
}
}

tool_enum! {
pub enum VcsEntityResource {
    Repository,
    Branch,
    Worktree,
    Assignment,
}
}

tool_schema! {
pub struct VcsEntityArgs {
    #[schemars(description = "Action: create, get, update, list, delete, release")]
    pub action: VcsEntityAction,

    #[schemars(description = "Resource: repository, branch, worktree, assignment")]
    pub resource: VcsEntityResource,

    #[schemars(description = "Resource ID (for get/update/delete/release)")]
    pub id: Option<String>,

    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    #[schemars(description = "Project ID (for repository listing)")]
    pub project_id: Option<String>,

    #[schemars(description = "Repository ID (for branch/worktree listing)")]
    pub repository_id: Option<String>,

    #[schemars(description = "Worktree ID (for assignment listing)")]
    pub worktree_id: Option<String>,

    #[schemars(
        description = "Data payload for create/update. phase: {name, status, order}; issue: {title, description?, status?, priority?}; dependency: {from_issue_id, to_issue_id, kind?}; decision: {title, rationale, impact?, status?}",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}
}

tool_enum! {
pub enum PlanEntityAction {
    Create,
    Get,
    Update,
    List,
    Delete,
}
}

tool_enum! {
pub enum PlanEntityResource {
    Plan,
    Version,
    Review,
}
}

tool_schema! {
pub struct PlanEntityArgs {
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: PlanEntityAction,

    #[schemars(description = "Resource: plan, version, review")]
    pub resource: PlanEntityResource,

    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    #[schemars(description = "Project ID (for plan listing)")]
    pub project_id: Option<String>,

    #[schemars(description = "Plan ID (for version listing)")]
    pub plan_id: Option<String>,

    #[schemars(description = "Plan version ID (for review listing)")]
    pub plan_version_id: Option<String>,

    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}
}

tool_enum! {
pub enum OrgEntityAction {
    Create,
    Get,
    Update,
    List,
    Delete,
}
}

tool_enum! {
pub enum OrgEntityResource {
    Org,
    User,
    Team,
    TeamMember,
    ApiKey,
}
}

tool_schema! {
pub struct OrgEntityArgs {
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: OrgEntityAction,
    #[schemars(description = "Resource: org, user, team, team_member, api_key")]
    pub resource: OrgEntityResource,
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,
    #[schemars(description = "Organization ID (for listing users/teams/api_keys)")]
    pub org_id: Option<String>,
    #[schemars(description = "Team ID (for listing members)")]
    pub team_id: Option<String>,
    #[schemars(description = "User ID (for removing team member)")]
    pub user_id: Option<String>,
    #[schemars(description = "Email (for user lookup by email)")]
    pub email: Option<String>,
    #[schemars(description = "Data payload for create/update (JSON object)")]
    #[schemars(with = "serde_json::Value")]
    pub data: Option<serde_json::Value>,
}
}

tool_enum! {
pub enum IssueEntityAction {
    Create,
    Get,
    Update,
    List,
    Delete,
}
}

tool_enum! {
pub enum IssueEntityResource {
    Issue,
    Comment,
    Label,
    LabelAssignment,
}
}

tool_schema! {
pub struct IssueEntityArgs {
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: IssueEntityAction,

    #[schemars(description = "Resource: issue, comment, label, label_assignment")]
    pub resource: IssueEntityResource,

    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    #[schemars(description = "Project ID (for issue/label listing)")]
    pub project_id: Option<String>,

    #[schemars(description = "Issue ID (for comment listing and label assignments)")]
    pub issue_id: Option<String>,

    #[schemars(description = "Label ID (for label unassignment)")]
    pub label_id: Option<String>,

    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}
}
