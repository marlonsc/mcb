use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::IssueEntityRepository;

/// SQLite-backed repository for issue, comment, and label entities.
pub struct SqliteIssueEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteIssueEntityRepository {
    /// Creates a new repository using the provided database executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    async fn query_one<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match self.executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

    async fn query_all<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Vec<T>>
    where
        F: Fn(&dyn SqlRow) -> Result<T>,
    {
        let rows = self.executor.query_all(sql, params).await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(
                convert(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode issue entity", e))?,
            );
        }
        Ok(result)
    }
}

fn row_to_issue(row: &dyn SqlRow) -> Result<ProjectIssue> {
    let labels_json = req_str(row, "labels")?;
    let labels = serde_json::from_str::<Vec<String>>(&labels_json)
        .map_err(|e| Error::memory_with_source("decode labels json", e))?;

    Ok(ProjectIssue {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        created_by: req_str(row, "created_by")?,
        phase_id: opt_str(row, "phase_id")?,
        title: req_str(row, "title")?,
        description: req_str(row, "description")?,
        issue_type: req_str(row, "issue_type")?
            .parse()
            .map_err(|e: String| Error::memory(e))?,
        status: req_str(row, "status")?
            .parse()
            .map_err(|e: String| Error::memory(e))?,
        priority: req_i64(row, "priority")? as i32,
        assignee: opt_str(row, "assignee")?,
        labels,
        estimated_minutes: opt_i64(row, "estimated_minutes")?,
        actual_minutes: opt_i64(row, "actual_minutes")?,
        notes: req_str(row, "notes")?,
        design: req_str(row, "design")?,
        parent_issue_id: opt_str(row, "parent_issue_id")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
        closed_at: opt_i64(row, "closed_at")?,
        closed_reason: req_str(row, "closed_reason")?,
    })
}

fn row_to_comment(row: &dyn SqlRow) -> Result<IssueComment> {
    Ok(IssueComment {
        id: req_str(row, "id")?,
        issue_id: req_str(row, "issue_id")?,
        author_id: req_str(row, "author_id")?,
        content: req_str(row, "content")?,
        created_at: req_i64(row, "created_at")?,
    })
}

fn row_to_label(row: &dyn SqlRow) -> Result<IssueLabel> {
    Ok(IssueLabel {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        name: req_str(row, "name")?,
        color: req_str(row, "color")?,
        created_at: req_i64(row, "created_at")?,
    })
}

fn req_str(row: &dyn SqlRow, col: &str) -> Result<String> {
    row.try_get_string(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

fn req_i64(row: &dyn SqlRow, col: &str) -> Result<i64> {
    row.try_get_i64(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

fn opt_str(row: &dyn SqlRow, col: &str) -> Result<Option<String>> {
    row.try_get_string(col)
}

fn opt_i64(row: &dyn SqlRow, col: &str) -> Result<Option<i64>> {
    row.try_get_i64(col)
}

fn opt_str_param(value: &Option<String>) -> SqlParam {
    match value {
        Some(v) => SqlParam::String(v.clone()),
        None => SqlParam::Null,
    }
}

fn opt_i64_param(value: Option<i64>) -> SqlParam {
    match value {
        Some(v) => SqlParam::I64(v),
        None => SqlParam::Null,
    }
}

#[async_trait]
impl IssueEntityRepository for SqliteIssueEntityRepository {
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let labels = serde_json::to_string(&issue.labels)
            .map_err(|e| Error::memory_with_source("encode labels json", e))?;
        self.executor
            .execute(
                "INSERT INTO project_issues (id, org_id, project_id, phase_id, title, description, issue_type, status, priority, assignee, labels, created_at, updated_at, closed_at, created_by, estimated_minutes, actual_minutes, notes, design, parent_issue_id, closed_reason) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(issue.id.clone()),
                    SqlParam::String(issue.org_id.clone()),
                    SqlParam::String(issue.project_id.clone()),
                    opt_str_param(&issue.phase_id),
                    SqlParam::String(issue.title.clone()),
                    SqlParam::String(issue.description.clone()),
                    SqlParam::String(issue.issue_type.as_str().to_string()),
                    SqlParam::String(issue.status.as_str().to_string()),
                    SqlParam::I64(i64::from(issue.priority)),
                    opt_str_param(&issue.assignee),
                    SqlParam::String(labels),
                    SqlParam::I64(issue.created_at),
                    SqlParam::I64(issue.updated_at),
                    opt_i64_param(issue.closed_at),
                    SqlParam::String(issue.created_by.clone()),
                    opt_i64_param(issue.estimated_minutes),
                    opt_i64_param(issue.actual_minutes),
                    SqlParam::String(issue.notes.clone()),
                    SqlParam::String(issue.design.clone()),
                    opt_str_param(&issue.parent_issue_id),
                    SqlParam::String(issue.closed_reason.clone()),
                ],
            )
            .await
    }

    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue> {
        self.query_one(
            "SELECT * FROM project_issues WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(id.to_string()),
            ],
            row_to_issue,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Issue {id}")))
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        self.query_all(
            "SELECT * FROM project_issues WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(project_id.to_string()),
            ],
            row_to_issue,
        )
        .await
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let labels = serde_json::to_string(&issue.labels)
            .map_err(|e| Error::memory_with_source("encode labels json", e))?;
        self.executor
            .execute(
                "UPDATE project_issues SET project_id = ?, phase_id = ?, title = ?, description = ?, issue_type = ?, status = ?, priority = ?, assignee = ?, labels = ?, updated_at = ?, closed_at = ?, created_by = ?, estimated_minutes = ?, actual_minutes = ?, notes = ?, design = ?, parent_issue_id = ?, closed_reason = ? WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(issue.project_id.clone()),
                    opt_str_param(&issue.phase_id),
                    SqlParam::String(issue.title.clone()),
                    SqlParam::String(issue.description.clone()),
                    SqlParam::String(issue.issue_type.as_str().to_string()),
                    SqlParam::String(issue.status.as_str().to_string()),
                    SqlParam::I64(i64::from(issue.priority)),
                    opt_str_param(&issue.assignee),
                    SqlParam::String(labels),
                    SqlParam::I64(issue.updated_at),
                    opt_i64_param(issue.closed_at),
                    SqlParam::String(issue.created_by.clone()),
                    opt_i64_param(issue.estimated_minutes),
                    opt_i64_param(issue.actual_minutes),
                    SqlParam::String(issue.notes.clone()),
                    SqlParam::String(issue.design.clone()),
                    opt_str_param(&issue.parent_issue_id),
                    SqlParam::String(issue.closed_reason.clone()),
                    SqlParam::String(issue.org_id.clone()),
                    SqlParam::String(issue.id.clone()),
                ],
            )
            .await
    }

    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM project_issues WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(id.to_string()),
                ],
            )
            .await
    }

    async fn create_comment(&self, comment: &IssueComment) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO issue_comments (id, issue_id, author_id, content, created_at) VALUES (?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(comment.id.clone()),
                    SqlParam::String(comment.issue_id.clone()),
                    SqlParam::String(comment.author_id.clone()),
                    SqlParam::String(comment.content.clone()),
                    SqlParam::I64(comment.created_at),
                ],
            )
            .await
    }

    async fn get_comment(&self, id: &str) -> Result<IssueComment> {
        self.query_one(
            "SELECT * FROM issue_comments WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_comment,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("IssueComment {id}")))
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        self.query_all(
            "SELECT * FROM issue_comments WHERE issue_id = ?",
            &[SqlParam::String(issue_id.to_string())],
            row_to_comment,
        )
        .await
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM issue_comments WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    async fn create_label(&self, label: &IssueLabel) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO issue_labels (id, org_id, project_id, name, color, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(label.id.clone()),
                    SqlParam::String(label.org_id.clone()),
                    SqlParam::String(label.project_id.clone()),
                    SqlParam::String(label.name.clone()),
                    SqlParam::String(label.color.clone()),
                    SqlParam::I64(label.created_at),
                ],
            )
            .await
    }

    async fn get_label(&self, id: &str) -> Result<IssueLabel> {
        self.query_one(
            "SELECT * FROM issue_labels WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_label,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("IssueLabel {id}")))
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        self.query_all(
            "SELECT * FROM issue_labels WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(project_id.to_string()),
            ],
            row_to_label,
        )
        .await
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM issue_labels WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    async fn assign_label(&self, assignment: &IssueLabelAssignment) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO issue_label_assignments (issue_id, label_id, created_at) VALUES (?, ?, ?)",
                &[
                    SqlParam::String(assignment.issue_id.clone()),
                    SqlParam::String(assignment.label_id.clone()),
                    SqlParam::I64(assignment.created_at),
                ],
            )
            .await
    }

    async fn unassign_label(&self, issue_id: &str, label_id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM issue_label_assignments WHERE issue_id = ? AND label_id = ?",
                &[
                    SqlParam::String(issue_id.to_string()),
                    SqlParam::String(label_id.to_string()),
                ],
            )
            .await
    }

    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>> {
        self.query_all(
            "SELECT l.* FROM issue_labels l INNER JOIN issue_label_assignments a ON a.label_id = l.id WHERE a.issue_id = ?",
            &[SqlParam::String(issue_id.to_string())],
            row_to_label,
        )
        .await
    }
}
