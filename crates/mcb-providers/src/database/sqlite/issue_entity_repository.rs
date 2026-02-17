//! `SQLite` Issue Entity Repository
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
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::{
    IssueCommentRegistry, IssueLabelAssignmentManager, IssueLabelRegistry, IssueRegistry,
};
use std::sync::Arc;

use super::row_convert;
use crate::utils::sqlite::query as query_helpers;
use crate::utils::sqlite::row::{opt_i64_param, opt_str_param};

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
            row_convert::row_to_issue,
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
            row_convert::row_to_issue,
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
            row_convert::row_to_comment,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("IssueComment {id}")))
    }

    async fn list_comments_by_issue(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM issue_comments WHERE issue_id = ?",
            &[SqlParam::String(issue_id.to_owned())],
            row_convert::row_to_comment,
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
            row_convert::row_to_label,
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
            row_convert::row_to_label,
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
            row_convert::row_to_label,
            "issue entity",
        )
        .await
    }
}
