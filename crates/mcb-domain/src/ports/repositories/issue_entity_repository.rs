use async_trait::async_trait;

use crate::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use crate::entities::project::ProjectIssue;
use crate::error::Result;

#[async_trait]
pub trait IssueEntityRepository: Send + Sync {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
    async fn get_issue(&self, org_id: &str, id: &str) -> Result<Option<ProjectIssue>>;
    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>>;
    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()>;

    async fn create_comment(&self, comment: &IssueComment) -> Result<()>;
    async fn get_comment(&self, id: &str) -> Result<Option<IssueComment>>;
    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>>;
    async fn delete_comment(&self, id: &str) -> Result<()>;

    async fn create_label(&self, label: &IssueLabel) -> Result<()>;
    async fn get_label(&self, id: &str) -> Result<Option<IssueLabel>>;
    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>>;
    async fn delete_label(&self, id: &str) -> Result<()>;

    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()>;
    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()>;
    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>>;
}
