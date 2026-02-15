//! Issue tracking schema entities (issues, comments, labels).

use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

/// Returns the table definitions (`project_issues`, `issue_comments`, `issue_labels`, etc).
#[must_use]
pub fn tables() -> Vec<TableDef> {
    vec![
        table!(
            "project_issues",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("project_id", Text),
                crate::col!("phase_id", Text, nullable),
                crate::col!("title", Text),
                crate::col!("description", Text),
                crate::col!("issue_type", Text),
                crate::col!("status", Text),
                crate::col!("priority", Integer),
                crate::col!("assignee", Text, nullable),
                crate::col!("labels", Text),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
                crate::col!("closed_at", Integer, nullable),
                crate::col!("created_by", Text),
                crate::col!("estimated_minutes", Integer, nullable),
                crate::col!("actual_minutes", Integer, nullable),
                crate::col!("notes", Text),
                crate::col!("design", Text),
                crate::col!("parent_issue_id", Text, nullable),
                crate::col!("closed_reason", Text),
            ]
        ),
        table!(
            "issue_comments",
            [
                crate::col!("id", Text, pk),
                crate::col!("issue_id", Text),
                crate::col!("author_id", Text),
                crate::col!("content", Text),
                crate::col!("created_at", Integer),
            ]
        ),
        table!(
            "issue_labels",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("project_id", Text),
                crate::col!("name", Text),
                crate::col!("color", Text),
                crate::col!("created_at", Integer),
            ]
        ),
        table!(
            "issue_label_assignments",
            [
                crate::col!("issue_id", Text, pk),
                crate::col!("label_id", Text, pk),
                crate::col!("created_at", Integer),
            ]
        ),
    ]
}

/// Returns the index definitions for issue tracking.
#[must_use]
pub fn indexes() -> Vec<IndexDef> {
    vec![
        index!("idx_issues_org", "project_issues", ["org_id"]),
        index!("idx_issues_project", "project_issues", ["project_id"]),
        index!("idx_issues_phase", "project_issues", ["phase_id"]),
        index!("idx_issues_status", "project_issues", ["status"]),
        index!("idx_issues_assignee", "project_issues", ["assignee"]),
        index!("idx_issues_parent", "project_issues", ["parent_issue_id"]),
        index!("idx_issue_comments_issue", "issue_comments", ["issue_id"]),
        index!("idx_issue_comments_author", "issue_comments", ["author_id"]),
        index!("idx_issue_labels_org", "issue_labels", ["org_id"]),
        index!("idx_issue_labels_project", "issue_labels", ["project_id"]),
        index!(
            "idx_issue_label_assignments_issue",
            "issue_label_assignments",
            ["issue_id"]
        ),
        index!(
            "idx_issue_label_assignments_label",
            "issue_label_assignments",
            ["label_id"]
        ),
    ]
}

/// Returns the foreign key definitions.
#[must_use]
pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "project_issues".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "project_issues".to_owned(),
            from_column: "project_id".to_owned(),
            to_table: "projects".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "project_issues".to_owned(),
            from_column: "created_by".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "project_issues".to_owned(),
            from_column: "parent_issue_id".to_owned(),
            to_table: "project_issues".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "issue_comments".to_owned(),
            from_column: "issue_id".to_owned(),
            to_table: "project_issues".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "issue_comments".to_owned(),
            from_column: "author_id".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "issue_labels".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "issue_labels".to_owned(),
            from_column: "project_id".to_owned(),
            to_table: "projects".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "issue_label_assignments".to_owned(),
            from_column: "issue_id".to_owned(),
            to_table: "project_issues".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "issue_label_assignments".to_owned(),
            from_column: "label_id".to_owned(),
            to_table: "issue_labels".to_owned(),
            to_column: "id".to_owned(),
        },
    ]
}

/// Returns the unique constraint definitions.
// TODO(qlty): Found 16 lines of similar code in 2 locations (mass = 56)
#[must_use]
pub fn unique_constraints() -> Vec<super::UniqueConstraintDef> {
    vec![
        super::UniqueConstraintDef {
            table: "issue_labels".to_owned(),
            columns: vec![
                "org_id".to_owned(),
                "project_id".to_owned(),
                "name".to_owned(),
            ],
        },
        super::UniqueConstraintDef {
            table: "issue_label_assignments".to_owned(),
            columns: vec!["issue_id".to_owned(), "label_id".to_owned()],
        },
    ]
}
