use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_crud_action_enum! {
/// CRUD actions for issue-related entity resources.
pub enum IssueEntityAction {
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

entity_args_schema! {
/// Arguments for issue-related entity operations.
pub struct IssueEntityArgs {
    action: IssueEntityAction,
    action_desc: "Action: create, get, update, list, delete",
    resource: IssueEntityResource,
    resource_desc: "Resource: issue, comment, label, label_assignment",
    /// Project ID (for issue/label listing)
    project_id: Option<String> => "Project ID (for issue/label listing)",
    /// Issue ID (for comment listing and label assignments)
    issue_id: Option<String> => "Issue ID (for comment listing and label assignments)",
    /// Label ID (for label unassignment)
    label_id: Option<String> => "Label ID (for label unassignment)",
}
}
