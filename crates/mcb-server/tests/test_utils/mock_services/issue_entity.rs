use async_trait::async_trait;
use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::repositories::IssueEntityRepository;

pub struct MockIssueEntityService;

impl MockIssueEntityService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockIssueEntityService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IssueEntityRepository for MockIssueEntityService {
    async fn create_issue(&self, _issue: &ProjectIssue) -> Result<()> {
        Ok(())
    }

    async fn get_issue(&self, _org_id: &str, _id: &str) -> Result<ProjectIssue> {
        Err(Error::not_found("not found"))
    }

    async fn list_issues(&self, _org_id: &str, _project_id: &str) -> Result<Vec<ProjectIssue>> {
        Ok(vec![])
    }

    async fn update_issue(&self, _issue: &ProjectIssue) -> Result<()> {
        Ok(())
    }

    async fn delete_issue(&self, _org_id: &str, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_comment(&self, _comment: &IssueComment) -> Result<()> {
        Ok(())
    }

    async fn get_comment(&self, _id: &str) -> Result<IssueComment> {
        Err(Error::not_found("not found"))
    }

    async fn list_comments_by_issue(&self, _issue_id: &str) -> Result<Vec<IssueComment>> {
        Ok(vec![])
    }

    async fn delete_comment(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_label(&self, _label: &IssueLabel) -> Result<()> {
        Ok(())
    }

    async fn get_label(&self, _id: &str) -> Result<IssueLabel> {
        Err(Error::not_found("not found"))
    }

    async fn list_labels(&self, _org_id: &str, _project_id: &str) -> Result<Vec<IssueLabel>> {
        Ok(vec![])
    }

    async fn delete_label(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn assign_label(&self, _assignment: &IssueLabelAssignment) -> Result<()> {
        Ok(())
    }

    async fn unassign_label(&self, _issue_id: &str, _label_id: &str) -> Result<()> {
        Ok(())
    }

    async fn list_labels_for_issue(&self, _issue_id: &str) -> Result<Vec<IssueLabel>> {
        Ok(vec![])
    }
}
