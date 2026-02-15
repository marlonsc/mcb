use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::args::macros::{tool_enum, tool_schema};

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

    /// Resource: issue, comment, label, `label_assignment`
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
