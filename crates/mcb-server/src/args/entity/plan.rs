//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_crud_action_enum! {
/// CRUD actions for plan-related entity resources.
pub enum PlanEntityAction {
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

entity_args_schema! {
/// Arguments for plan-related entity operations.
pub struct PlanEntityArgs {
    action: PlanEntityAction,
    action_desc: "Action: create, get, update, list, delete",
    resource: PlanEntityResource,
    resource_desc: "Resource: plan, version, review",
    /// Project ID (for plan listing)
    project_id: Option<String> => "Project ID (for plan listing)",
    /// Plan ID (for version listing)
    plan_id: Option<String> => "Plan ID (for version listing)",
    /// Plan version ID (for review listing)
    plan_version_id: Option<String> => "Plan version ID (for review listing)",
}
}
