use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

/// Performs the tables operation.
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

/// Performs the indexes operation.
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

/// Performs the foreign keys operation.
pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "project_issues".to_string(),
            from_column: "org_id".to_string(),
            to_table: "organizations".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_issues".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_issues".to_string(),
            from_column: "created_by".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_issues".to_string(),
            from_column: "parent_issue_id".to_string(),
            to_table: "project_issues".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "issue_comments".to_string(),
            from_column: "issue_id".to_string(),
            to_table: "project_issues".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "issue_comments".to_string(),
            from_column: "author_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "issue_labels".to_string(),
            from_column: "org_id".to_string(),
            to_table: "organizations".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "issue_labels".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "issue_label_assignments".to_string(),
            from_column: "issue_id".to_string(),
            to_table: "project_issues".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "issue_label_assignments".to_string(),
            from_column: "label_id".to_string(),
            to_table: "issue_labels".to_string(),
            to_column: "id".to_string(),
        },
    ]
}

/// Performs the unique constraints operation.
pub fn unique_constraints() -> Vec<super::UniqueConstraintDef> {
    vec![
        super::UniqueConstraintDef {
            table: "issue_labels".to_string(),
            columns: vec![
                "org_id".to_string(),
                "project_id".to_string(),
                "name".to_string(),
            ],
        },
        super::UniqueConstraintDef {
            table: "issue_label_assignments".to_string(),
            columns: vec!["issue_id".to_string(), "label_id".to_string()],
        },
    ]
}
