//! Issue Entity Repository Port
//!
//! # Overview
//! Defines the interface for persisting issue-related entities including issues,
//! comments, labels, and assignments.
use async_trait::async_trait;

use crate::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use crate::entities::project::ProjectIssue;
use crate::error::Result;

#[async_trait]
/// Defines behavior for `IssueEntityRepository`.
#[async_trait]
/// Registry for issues.
pub trait IssueRegistry: Send + Sync {
    /// Performs the create issue operation.
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Performs the get issue operation.
    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue>;
    /// Performs the list issues operation.
    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>>;
    /// Performs the update issue operation.
    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Performs the delete issue operation.
    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()>;
}

#[async_trait]
/// Registry for issue comments.
pub trait IssueCommentRegistry: Send + Sync {
    /// Performs the create comment operation.
    async fn create_comment(&self, comment: &IssueComment) -> Result<()>;
    /// Performs the get comment operation.
    async fn get_comment(&self, id: &str) -> Result<IssueComment>;
    /// Performs the list comments by issue operation.
    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>>;
    /// Performs the delete comment operation.
    async fn delete_comment(&self, id: &str) -> Result<()>;
}

#[async_trait]
/// Registry for issue labels.
pub trait IssueLabelRegistry: Send + Sync {
    /// Performs the create label operation.
    async fn create_label(&self, label: &IssueLabel) -> Result<()>;
    /// Performs the get label operation.
    async fn get_label(&self, id: &str) -> Result<IssueLabel>;
    /// Performs the list labels operation.
    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>>;
    /// Performs the delete label operation.
    async fn delete_label(&self, id: &str) -> Result<()>;
}

#[async_trait]
/// Manager for issue label assignments.
pub trait IssueLabelAssignmentManager: Send + Sync {
    /// Performs the assign label operation.
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()>;
    /// Performs the unassign label operation.
    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()>;
    /// Performs the list labels for issue operation.
    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>>;
}

/// Aggregate trait for issue entity management.
pub trait IssueEntityRepository:
    IssueRegistry
    + IssueCommentRegistry
    + IssueLabelRegistry
    + IssueLabelAssignmentManager
    + Send
    + Sync
{
}

impl<T> IssueEntityRepository for T where
    T: IssueRegistry
        + IssueCommentRegistry
        + IssueLabelRegistry
        + IssueLabelAssignmentManager
        + Send
        + Sync
{
}
