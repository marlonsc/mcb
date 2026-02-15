use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::args::macros::{tool_enum, tool_schema};

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
