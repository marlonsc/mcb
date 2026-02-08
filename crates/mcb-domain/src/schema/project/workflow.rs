//! Workflow schema elements for project phases, issues, dependencies, and decisions.

use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

/// Returns the table definitions.
pub fn tables() -> Vec<TableDef> {
    vec![
        crate::table!(
            "project_phases",
            [
                crate::col!("id", Text, pk),
                crate::col!("project_id", Text),
                crate::col!("name", Text),
                crate::col!("description", Text),
                crate::col!("sequence", Integer),
                crate::col!("status", Text),
                crate::col!("started_at", Integer, nullable),
                crate::col!("completed_at", Integer, nullable),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
            ]
        ),
        crate::table!(
            "project_issues",
            [
                crate::col!("id", Text, pk),
                crate::col!("project_id", Text),
                crate::col!("phase_id", Text, nullable),
                crate::col!("title", Text),
                crate::col!("description", Text),
                crate::col!("issue_type", Text),
                crate::col!("status", Text),
                crate::col!("priority", Integer),
                crate::col!("assignee", Text, nullable),
                crate::col!("labels", Text, nullable),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
                crate::col!("closed_at", Integer, nullable),
            ]
        ),
        crate::table!(
            "project_dependencies",
            [
                crate::col!("id", Text, pk),
                crate::col!("from_issue_id", Text),
                crate::col!("to_issue_id", Text),
                crate::col!("dependency_type", Text),
                crate::col!("created_at", Integer),
            ]
        ),
        crate::table!(
            "project_decisions",
            [
                crate::col!("id", Text, pk),
                crate::col!("project_id", Text),
                crate::col!("issue_id", Text, nullable),
                crate::col!("title", Text),
                crate::col!("context", Text),
                crate::col!("decision", Text),
                crate::col!("consequences", Text),
                crate::col!("created_at", Integer),
            ]
        ),
    ]
}

/// Returns the index definitions.
pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_project_phases_project",
            "project_phases",
            ["project_id"]
        ),
        crate::index!("idx_project_phases_status", "project_phases", ["status"]),
        crate::index!(
            "idx_project_issues_project",
            "project_issues",
            ["project_id"]
        ),
        crate::index!("idx_project_issues_phase", "project_issues", ["phase_id"]),
        crate::index!("idx_project_issues_status", "project_issues", ["status"]),
        crate::index!(
            "idx_project_issues_priority",
            "project_issues",
            ["priority"]
        ),
        crate::index!(
            "idx_project_dependencies_from",
            "project_dependencies",
            ["from_issue_id"]
        ),
        crate::index!(
            "idx_project_dependencies_to",
            "project_dependencies",
            ["to_issue_id"]
        ),
        crate::index!(
            "idx_project_decisions_project",
            "project_decisions",
            ["project_id"]
        ),
        crate::index!(
            "idx_project_decisions_issue",
            "project_decisions",
            ["issue_id"]
        ),
    ]
}

/// Returns the foreign key definitions.
pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "project_phases".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
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
            from_column: "phase_id".to_string(),
            to_table: "project_phases".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_dependencies".to_string(),
            from_column: "from_issue_id".to_string(),
            to_table: "project_issues".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_dependencies".to_string(),
            from_column: "to_issue_id".to_string(),
            to_table: "project_issues".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_decisions".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "project_decisions".to_string(),
            from_column: "issue_id".to_string(),
            to_table: "project_issues".to_string(),
            to_column: "id".to_string(),
        },
    ]
}
