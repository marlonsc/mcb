//! Issue registry and label management implementations.
//!
//! Implements `IssueRegistry`, `IssueCommentRegistry`, `IssueLabelRegistry`,
//! and `IssueLabelAssignmentManager` for managing issues, comments, labels,
//! and label assignments.

use super::*;

#[async_trait]
impl IssueRegistry for SeaOrmEntityRepository {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_repo_insert!(self.db(), project_issue, issue, "create project issue")
    }

    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        sea_repo_get_filtered!(self.db(), project_issue, ProjectIssue, "Issue", id, "get project issue", project_issue::Column::OrgId => org_id)
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        sea_repo_list!(self.db(), project_issue, ProjectIssue, "list project issues", project_issue::Column::OrgId => org_id, project_issue::Column::ProjectId => project_id)
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        sea_repo_update!(self.db(), project_issue, issue, "update project issue")
    }

    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        sea_repo_delete_filtered!(self.db(), project_issue, id, "delete project issue", project_issue::Column::OrgId => org_id)
    }
}

#[async_trait]
impl IssueCommentRegistry for SeaOrmEntityRepository {
    async fn create_comment(&self, comment: &IssueComment) -> Result<()> {
        sea_repo_insert!(self.db(), issue_comment, comment, "create issue comment")
    }

    async fn get_comment(&self, id: &str) -> Result<IssueComment> {
        sea_repo_get!(
            self.db(),
            issue_comment,
            IssueComment,
            "IssueComment",
            id,
            "get issue comment"
        )
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        sea_repo_list!(self.db(), issue_comment, IssueComment, "list issue comments", issue_comment::Column::IssueId => issue_id)
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), issue_comment, id, "delete issue comment")
    }
}

#[async_trait]
impl IssueLabelRegistry for SeaOrmEntityRepository {
    async fn create_label(&self, label: &IssueLabel) -> Result<()> {
        sea_repo_insert!(self.db(), issue_label, label, "create issue label")
    }

    async fn get_label(&self, id: &str) -> Result<IssueLabel> {
        sea_repo_get!(
            self.db(),
            issue_label,
            IssueLabel,
            "IssueLabel",
            id,
            "get issue label"
        )
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        sea_repo_list!(self.db(), issue_label, IssueLabel, "list issue labels", issue_label::Column::OrgId => org_id, issue_label::Column::ProjectId => project_id)
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), issue_label, id, "delete issue label")
    }
}

#[async_trait]
impl IssueLabelAssignmentManager for SeaOrmEntityRepository {
    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()> {
        sea_repo_insert!(
            self.db(),
            issue_label_assignment,
            assignment,
            "assign issue label"
        )
    }

    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()> {
        sea_repo_delete!(
            self.db(),
            issue_label_assignment,
            (issue_id.to_owned(), label_id.to_owned()),
            "unassign issue label"
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
