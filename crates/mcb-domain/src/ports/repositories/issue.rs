//! Issue repository ports.

use async_trait::async_trait;

use crate::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use crate::entities::project::ProjectIssue;
use crate::error::Result;

define_crud_port! {
    /// Registry for issues.
    pub trait IssueRegistry {
        entity: ProjectIssue,
        create: create_issue,
        get: get_issue(org_id, id),
        list: list_issues(org_id, project_id),
        update: update_issue,
        delete: delete_issue(org_id, id),
    }
}

define_crud_port! {
    /// Registry for issue comments.
    pub trait IssueCommentRegistry {
        entity: IssueComment,
        create: create_comment,
        get: get_comment(id),
        list: list_comments_by_issue(issue_id),
        delete: delete_comment(id),
    }
}

define_crud_port! {
    /// Registry for issue labels.
    pub trait IssueLabelRegistry {
        entity: IssueLabel,
        create: create_label,
        get: get_label(id),
        list: list_labels(org_id, project_id),
        delete: delete_label(id),
    }
}

/// Manager for issue label assignments.
#[async_trait]
pub trait IssueLabelAssignmentManager: Send + Sync {
    /// Assign a label to an issue.
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()>;
    /// Unassign a label from an issue.
    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()>;
    /// List labels for an issue.
    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>>;
}

define_aggregate! {
    /// Aggregate trait for issue entity management.
    #[async_trait]
    pub trait IssueEntityRepository = IssueRegistry + IssueCommentRegistry + IssueLabelRegistry + IssueLabelAssignmentManager;
}
