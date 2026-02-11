//! Issue entity service implementation.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::IssueEntityRepository;
use mcb_domain::ports::services::IssueEntityServiceInterface;

/// Application-layer service for issue entity CRUD operations.
pub struct IssueEntityServiceImpl {
    repository: Arc<dyn IssueEntityRepository>,
}

impl IssueEntityServiceImpl {
    /// Create a new [`IssueEntityServiceImpl`] backed by the given repository.
    pub fn new(repository: Arc<dyn IssueEntityRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl IssueEntityServiceInterface for IssueEntityServiceImpl {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        self.repository.create_issue(issue).await
    }

    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        self.repository.get_issue(org_id, id).await
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        self.repository.list_issues(org_id, project_id).await
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        self.repository.update_issue(issue).await
    }

    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        self.repository.delete_issue(org_id, id).await
    }

    async fn create_comment(&self, comment: &IssueComment) -> Result<()> {
        self.repository.create_comment(comment).await
    }

    async fn get_comment(&self, id: &str) -> Result<IssueComment> {
        self.repository.get_comment(id).await
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        self.repository.list_comments_by_issue(issue_id).await
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        self.repository.delete_comment(id).await
    }

    async fn create_label(&self, label: &IssueLabel) -> Result<()> {
        self.repository.create_label(label).await
    }

    async fn get_label(&self, id: &str) -> Result<IssueLabel> {
        self.repository.get_label(id).await
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        self.repository.list_labels(org_id, project_id).await
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        self.repository.delete_label(id).await
    }

    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()> {
        self.repository.assign_label(assignment).await
    }

    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()> {
        self.repository.unassign_label(issue_id, label_id).await
    }

    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>> {
        self.repository.list_labels_for_issue(issue_id).await
    }
}
