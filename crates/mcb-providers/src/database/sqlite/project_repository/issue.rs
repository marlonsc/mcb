use std::sync::Arc;

use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::IssueFilter;
use tracing::debug;

use crate::database::sqlite::row_convert;

pub(crate) async fn create_issue(
    executor: &Arc<dyn DatabaseExecutor>,
    issue: &ProjectIssue,
) -> Result<()> {
    let labels_json = serde_json::to_string(&issue.labels)
        .map_err(|e| Error::memory_with_source("serialize issue labels", e))?;

    let params = [
        SqlParam::String(issue.id.clone()),
        SqlParam::String(issue.project_id.clone()),
        issue
            .phase_id
            .as_ref()
            .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        SqlParam::String(issue.title.clone()),
        SqlParam::String(issue.description.clone()),
        SqlParam::String(issue.issue_type.as_str().to_string()),
        SqlParam::String(issue.status.as_str().to_string()),
        SqlParam::I64(issue.priority as i64),
        issue
            .assignee
            .as_ref()
            .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        SqlParam::String(labels_json),
        SqlParam::I64(issue.created_at),
        SqlParam::I64(issue.updated_at),
        issue.closed_at.map_or(SqlParam::Null, SqlParam::I64),
    ];

    executor
        .execute(
            r"
            INSERT INTO project_issues (
                id, project_id, phase_id, title, description, issue_type, status,
                priority, assignee, labels, created_at, updated_at, closed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            &params,
        )
        .await?;

    debug!("Created issue: {}", issue.id);
    Ok(())
}

pub(crate) async fn get_issue(
    executor: &Arc<dyn DatabaseExecutor>,
    id: &str,
) -> Result<Option<ProjectIssue>> {
    let row = executor
        .query_one(
            "SELECT * FROM project_issues WHERE id = ?",
            &[SqlParam::String(id.to_string())],
        )
        .await?;

    match row {
        Some(r) => Ok(Some(
            row_convert::row_to_project_issue(r.as_ref())
                .map_err(|e| Error::memory_with_source("decode project issue row", e))?,
        )),
        None => Ok(None),
    }
}

pub(crate) async fn update_issue(
    executor: &Arc<dyn DatabaseExecutor>,
    issue: &ProjectIssue,
) -> Result<()> {
    let labels_json = serde_json::to_string(&issue.labels)
        .map_err(|e| Error::memory_with_source("serialize issue labels", e))?;

    let params = [
        issue
            .phase_id
            .as_ref()
            .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        SqlParam::String(issue.title.clone()),
        SqlParam::String(issue.description.clone()),
        SqlParam::String(issue.issue_type.as_str().to_string()),
        SqlParam::String(issue.status.as_str().to_string()),
        SqlParam::I64(issue.priority as i64),
        issue
            .assignee
            .as_ref()
            .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        SqlParam::String(labels_json),
        SqlParam::I64(issue.updated_at),
        issue.closed_at.map_or(SqlParam::Null, SqlParam::I64),
        SqlParam::String(issue.id.clone()),
    ];

    executor
        .execute(
            r"
            UPDATE project_issues
            SET phase_id = ?, title = ?, description = ?, issue_type = ?, status = ?,
                priority = ?, assignee = ?, labels = ?, updated_at = ?, closed_at = ?
            WHERE id = ?
            ",
            &params,
        )
        .await?;

    debug!("Updated issue: {}", issue.id);
    Ok(())
}

pub(crate) async fn list_issues(
    executor: &Arc<dyn DatabaseExecutor>,
    project_id: &str,
) -> Result<Vec<ProjectIssue>> {
    super::helpers::query_and_convert_all(
        executor,
        "SELECT * FROM project_issues WHERE project_id = ? ORDER BY created_at DESC",
        &[SqlParam::String(project_id.to_string())],
        row_convert::row_to_project_issue,
    )
    .await
}

pub(crate) async fn filter_issues(
    executor: &Arc<dyn DatabaseExecutor>,
    filter: &IssueFilter,
) -> Result<Vec<ProjectIssue>> {
    let mut sql = String::from("SELECT * FROM project_issues WHERE 1=1");
    let mut params: Vec<SqlParam> = Vec::new();

    if let Some(project_id) = &filter.project_id {
        sql.push_str(" AND project_id = ?");
        params.push(SqlParam::String(project_id.clone()));
    }
    if let Some(phase_id) = &filter.phase_id {
        sql.push_str(" AND phase_id = ?");
        params.push(SqlParam::String(phase_id.clone()));
    }
    if let Some(issue_type) = &filter.issue_type {
        sql.push_str(" AND issue_type = ?");
        params.push(SqlParam::String(issue_type.as_str().to_string()));
    }
    if let Some(status) = &filter.status {
        sql.push_str(" AND status = ?");
        params.push(SqlParam::String(status.as_str().to_string()));
    }
    if let Some(priority) = filter.priority {
        sql.push_str(" AND priority = ?");
        params.push(SqlParam::I64(priority as i64));
    }
    if let Some(assignee) = &filter.assignee {
        sql.push_str(" AND assignee = ?");
        params.push(SqlParam::String(assignee.clone()));
    }
    if let Some(label) = &filter.label {
        sql.push_str(" AND labels LIKE ?");
        params.push(SqlParam::String(format!("%{}%", label)));
    }

    sql.push_str(" ORDER BY priority ASC, created_at DESC");
    if let Some(limit) = filter.limit {
        sql.push_str(" LIMIT ?");
        params.push(SqlParam::I64(limit as i64));
    }

    let rows = executor.query_all(&sql, &params).await?;
    let mut issues = Vec::with_capacity(rows.len());
    for row in rows {
        issues.push(
            row_convert::row_to_project_issue(row.as_ref())
                .map_err(|e| Error::memory_with_source("decode project issue row", e))?,
        );
    }
    Ok(issues)
}
