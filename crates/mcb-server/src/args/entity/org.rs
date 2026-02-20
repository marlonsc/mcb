//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

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
    /// Resource: org, user, team, `team_member`, `api_key`
    #[schemars(description = "Resource: org, user, team, team_member, api_key")]
    pub resource: OrgEntityResource,
    /// Resource ID (for get/update/delete)
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,
    /// Organization ID (for listing `users/teams/api_keys`)
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
