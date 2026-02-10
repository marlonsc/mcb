//! Issue-related entities used by the `issue_entity` domain.

use serde::{Deserialize, Serialize};

/// A comment authored on a project issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    /// Unique identifier for the comment.
    pub id: String,
    /// Parent issue identifier.
    pub issue_id: String,
    /// Author user identifier.
    pub author_id: String,
    /// Free-form comment body.
    pub content: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
}

/// A reusable issue label scoped to org and project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLabel {
    /// Unique identifier for the label.
    pub id: String,
    /// Organization identifier (tenant isolation).
    pub org_id: String,
    /// Project identifier.
    pub project_id: String,
    /// Label display name.
    pub name: String,
    /// Label color value.
    pub color: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
}

/// Junction entity assigning labels to issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLabelAssignment {
    /// Issue identifier.
    pub issue_id: String,
    /// Label identifier.
    pub label_id: String,
    /// Assignment creation timestamp (Unix epoch).
    pub created_at: i64,
}
