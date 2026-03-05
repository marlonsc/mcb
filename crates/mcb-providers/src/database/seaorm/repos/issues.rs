//! Issue registry and label management implementations.
//!
//! Implements `IssueRegistry`, `IssueCommentRegistry`, `IssueLabelRegistry`,
//! and `IssueLabelAssignmentManager` for managing issues, comments, labels,
//! and label assignments.

use super::*;

sea_impl_crud_scoped!(IssueRegistry for SeaOrmEntityRepository { db: db,
    entity: project_issue, domain: ProjectIssue, label: "Issue",
    scope_col: project_issue::Column::OrgId,
    create: create_issue(issue),
    get: get_issue,
    list: list_issues(project_issue::Column::ProjectId => project_id),
    update: update_issue(issue),
    delete: delete_issue
});

sea_impl_crud!(IssueCommentRegistry for SeaOrmEntityRepository { db: db,
    entity: issue_comment, domain: IssueComment, label: "IssueComment",
    create: create_comment(comment),
    get: get_comment(id),
    list: list_comments_by_issue(issue_comment::Column::IssueId => issue_id),
    delete: delete_comment(id)
});

sea_impl_crud!(IssueLabelRegistry for SeaOrmEntityRepository { db: db,
    entity: issue_label, domain: IssueLabel, label: "IssueLabel",
    create: create_label(label),
    get: get_label(id),
    list: list_labels(issue_label::Column::OrgId => org_id, issue_label::Column::ProjectId => project_id),
    delete: delete_label(id)
});

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
