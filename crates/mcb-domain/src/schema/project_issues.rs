use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
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
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_issues_org", "project_issues", ["org_id"]),
        crate::index!("idx_issues_project", "project_issues", ["project_id"]),
        crate::index!("idx_issues_phase", "project_issues", ["phase_id"]),
        crate::index!("idx_issues_status", "project_issues", ["status"]),
        crate::index!("idx_issues_assignee", "project_issues", ["assignee"]),
        crate::index!("idx_issues_parent", "project_issues", ["parent_issue_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("project_issues", "org_id", "organizations", "id"),
        crate::fk!("project_issues", "project_id", "projects", "id"),
        crate::fk!("project_issues", "created_by", "users", "id"),
        crate::fk!("project_issues", "parent_issue_id", "project_issues", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
