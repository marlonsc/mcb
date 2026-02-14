use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

tool_enum! {
pub enum ProjectAction {
    Create,
    Get,
    Update,
    List,
    Delete,
}
}

tool_enum! {
pub enum ProjectResource {
    Project,
    Phase,
    Issue,
    Dependency,
    Decision,
}
}

tool_schema! {
pub struct ProjectArgs {
    #[schemars(description = "Action: create, update, list, delete")]
    pub action: ProjectAction,

    #[schemars(description = "Resource type: phase, issue, dependency, decision")]
    pub resource: ProjectResource,

    #[schemars(description = "Project ID")]
    pub project_id: String,

    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    #[schemars(
        description = "Additional filters for list action",
        with = "serde_json::Value"
    )]
    pub filters: Option<serde_json::Value>,
}
}
