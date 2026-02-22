//! Issue-related entities used by the `issue_entity` domain.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity! {
    /// A comment authored on a project issue.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct IssueComment { id, created_at } {
        /// Issue identifier this comment belongs to.
        pub issue_id: String,
        /// User identifier of the commentator.
        pub author_id: String,
        /// Markdown content of the comment.
        pub content: String,
    }
}

crate::define_entity! {
    /// A reusable issue label scoped to org and project.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct IssueLabel { id, org_id, project_id, created_at } {
        /// Display name of the label.
        pub name: String,
        /// CSS color code or hex string.
        pub color: String,
    }
}

use crate::value_objects::ids::IssueLabelAssignmentId;

/// Junction entity assigning labels to issues.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct IssueLabelAssignment {
    /// Unique identifier for the assignment (composite of `issue_id:label_id`).
    #[serde(default)]
    pub id: IssueLabelAssignmentId,
    /// Issue identifier.
    pub issue_id: String,
    /// Label identifier.
    pub label_id: String,
    /// Assignment creation timestamp (Unix epoch).
    pub created_at: i64,
}
