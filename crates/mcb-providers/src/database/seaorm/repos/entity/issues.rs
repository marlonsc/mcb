//! Issue registry and label management implementations.
//!
//! Implements `IssueRegistry`, `IssueCommentRegistry`, `IssueLabelRegistry`,
//! and `IssueLabelAssignmentManager` for managing issues, comments, labels,
//! and label assignments.

use super::*;

#[async_trait]
impl IssueRegistry for SeaOrmEntityRepository {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_insert!(self, project_issue, issue)
    }

    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        sea_get_filtered!(self, project_issue, ProjectIssue, "Issue", id, project_issue::Column::OrgId => org_id)
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        sea_list!(self, project_issue, ProjectIssue, project_issue::Column::OrgId => org_id, project_issue::Column::ProjectId => project_id)
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_update!(self, project_issue, issue)
    }

    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        sea_delete_filtered!(self, project_issue, id, project_issue::Column::OrgId => org_id)
    }
}

#[async_trait]
impl IssueCommentRegistry for SeaOrmEntityRepository {
    async fn create_comment(&self, comment: &IssueComment) -> Result<()> {
        sea_insert!(self, issue_comment, comment)
    }

    async fn get_comment(&self, id: &str) -> Result<IssueComment> {
        sea_get!(self, issue_comment, IssueComment, "IssueComment", id)
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        sea_list!(self, issue_comment, IssueComment, issue_comment::Column::IssueId => issue_id)
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        sea_delete!(self, issue_comment, id)
    }
}

#[async_trait]
impl IssueLabelRegistry for SeaOrmEntityRepository {
    async fn create_label(&self, label: &IssueLabel) -> Result<()> {
        sea_insert!(self, issue_label, label)
    }

    async fn get_label(&self, id: &str) -> Result<IssueLabel> {
        sea_get!(self, issue_label, IssueLabel, "IssueLabel", id)
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        sea_list!(self, issue_label, IssueLabel, issue_label::Column::OrgId => org_id, issue_label::Column::ProjectId => project_id)
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        sea_delete!(self, issue_label, id)
    }
}

#[async_trait]
impl IssueLabelAssignmentManager for SeaOrmEntityRepository {
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()> {
        sea_insert!(self, issue_label_assignment, assignment)
    }

    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()> {
        sea_delete!(
            self,
            issue_label_assignment,
            (issue_id.to_owned(), label_id.to_owned())
        )
    }

    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>> {
        let assignments = issue_label_assignment::Entity::find()
            .filter(issue_label_assignment::Column::IssueId.eq(issue_id))
            .all(self.db())
            .await
            .map_err(db_err)?;

        if assignments.is_empty() {
            return Ok(vec![]);
        }

        let label_ids: Vec<String> = assignments.into_iter().map(|a| a.label_id).collect();
        let labels = issue_label::Entity::find()
            .filter(issue_label::Column::Id.is_in(label_ids))
            .all(self.db())
            .await
            .map_err(db_err)?;

        Ok(labels.into_iter().map(IssueLabel::from).collect())
    }
}
