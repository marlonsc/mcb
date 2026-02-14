//! SQLite Issue Entity Repository
//!
//! # Overview
//! The `SqliteIssueEntityRepository` provides persistence for the Issue Tracking domain.
//! It manages the comprehensive lifecycle of issues, including commenting, labeling,
//! and metadata tracking (assignments, estimations, status).
//!
//! # Responsibilities
//! - **Issue Management**: CRUD for `ProjectIssue`, including complex fields like `labels` (JSON).
//! - **Discussion Thread**: Managing `IssueComment` records linked to issues.
//! - **Taxonomy**: Creation and assignment of `IssueLabel` entities.
//! - **Relations**: Managing many-to-many relationships (e.g., issue-label assignments).

use async_trait::async_trait;
use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::entities::project::{IssueStatus, IssueType};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::issue_entity_repository::{
    IssueCommentRegistry, IssueLabelAssignmentManager, IssueLabelRegistry, IssueRegistry,
};
use std::str::FromStr;
use std::sync::Arc;

use super::query_helpers;
use super::row_helpers::{opt_i64, opt_i64_param, opt_str, opt_str_param, req_i64, req_str};

/// SQLite-based implementation of `IssueEntityRepository`.
///
/// Handles storage for `project_issues`, `issue_comments`, `issue_labels`, and
/// `issue_label_assignments` tables.
pub struct SqliteIssueEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteIssueEntityRepository {
    /// Creates a new repository using the provided database executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

// TODO(architecture): Move row conversion logic to shared row_convert module.
// This file implements its own conversion helpers instead of using the shared infrastructure.
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
        issue_type: IssueType::from_str(&req_str(row, "issue_type")?)
            .map_err(|e| Error::memory(e.to_string()))?,
        status: IssueStatus::from_str(&req_str(row, "status")?)
            .map_err(|e| Error::memory(e.to_string()))?,
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

#[async_trait]
impl IssueRegistry for SqliteIssueEntityRepository {
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
                    SqlParam::String(issue.issue_type.as_str().to_owned()),
                    SqlParam::String(issue.status.as_str().to_owned()),
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
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM project_issues WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(id.to_owned()),
            ],
            row_to_issue,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Issue {id}")))
    }

    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM project_issues WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(project_id.to_owned()),
            ],
            row_to_issue,
            "issue entity",
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
                    SqlParam::String(issue.issue_type.as_str().to_owned()),
                    SqlParam::String(issue.status.as_str().to_owned()),
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
                    SqlParam::String(org_id.to_owned()),
                    SqlParam::String(id.to_owned()),
                ],
            )
            .await
    }
}

#[async_trait]
impl IssueCommentRegistry for SqliteIssueEntityRepository {
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
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM issue_comments WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_comment,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("IssueComment {id}")))
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM issue_comments WHERE issue_id = ?",
            &[SqlParam::String(issue_id.to_owned())],
            row_to_comment,
            "issue entity",
        )
        .await
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM issue_comments WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
}

#[async_trait]
impl IssueLabelRegistry for SqliteIssueEntityRepository {
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
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM issue_labels WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_label,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("IssueLabel {id}")))
    }

    async fn list_labels(&self, org_id: &str, project_id: &str) -> Result<Vec<IssueLabel>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM issue_labels WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(project_id.to_owned()),
            ],
            row_to_label,
            "issue entity",
        )
        .await
    }

    async fn delete_label(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM issue_labels WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
}

#[async_trait]
impl IssueLabelAssignmentManager for SqliteIssueEntityRepository {
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
                    SqlParam::String(issue_id.to_owned()),
                    SqlParam::String(label_id.to_owned()),
                ],
            )
            .await
    }

    async fn list_labels_for_issue(&self, issue_id: &str) -> Result<Vec<IssueLabel>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT l.* FROM issue_labels l INNER JOIN issue_label_assignments a ON a.label_id = l.id WHERE a.issue_id = ?",
            &[SqlParam::String(issue_id.to_owned())],
            row_to_label,
            "issue entity",
        )
        .await
    }
}
