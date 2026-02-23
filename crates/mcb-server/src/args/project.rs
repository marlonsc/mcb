//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_enum! {
/// Actions available for project resource management
pub enum ProjectAction {
    /// Create a new resource.
    Create,
    /// Get an existing resource.
    Get,
    /// Update an existing resource.
    Update,
    /// List resources.
    List,
    /// Delete a resource.
    Delete,
}
}

tool_enum! {
/// Types of project resources that can be managed
pub enum ProjectResource {
    /// Project metadata.
    Project,
    /// Project phase.
    Phase,
    /// Project issue.
    Issue,
    /// Issue dependency.
    Dependency,
    /// Project decision.
    Decision,
}
}

tool_schema! {
/// Arguments for project resource management operations
pub struct ProjectArgs {
    /// Action: create, update, list, delete
    #[schemars(description = "Action: create, update, list, delete")]
    pub action: ProjectAction,

    /// Resource type: phase, issue, dependency, decision
    #[schemars(description = "Resource type: phase, issue, dependency, decision")]
    pub resource: ProjectResource,

    /// Project ID
    #[schemars(description = "Project ID")]
    pub project_id: String,

    /// Data payload for create/update (JSON object)
    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    /// Additional filters for list action
    #[schemars(
        description = "Additional filters for list action",
        with = "serde_json::Value"
    )]
    pub filters: Option<serde_json::Value>,
}
}
