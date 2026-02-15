use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

/// Performs the tables operation.
#[must_use]
pub fn tables() -> Vec<TableDef> {
    vec![
        table!(
            "plans",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("project_id", Text),
                crate::col!("title", Text),
                crate::col!("description", Text),
                crate::col!("status", Text),
                crate::col!("created_by", Text),
                crate::col!("created_at", Integer),
                crate::col!("updated_at", Integer),
            ]
        ),
        table!(
            "plan_versions",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("plan_id", Text),
                crate::col!("version_number", Integer),
                crate::col!("content_json", Text),
                crate::col!("change_summary", Text),
                crate::col!("created_by", Text),
                crate::col!("created_at", Integer),
            ]
        ),
        table!(
            "plan_reviews",
            [
                crate::col!("id", Text, pk),
                crate::col!("org_id", Text),
                crate::col!("plan_version_id", Text),
                crate::col!("reviewer_id", Text),
                crate::col!("verdict", Text),
                crate::col!("feedback", Text),
                crate::col!("created_at", Integer),
            ]
        ),
    ]
}

/// Performs the indexes operation.
#[must_use]
pub fn indexes() -> Vec<IndexDef> {
    vec![
        index!("idx_plans_org", "plans", ["org_id"]),
        index!("idx_plans_project", "plans", ["project_id"]),
        index!("idx_plans_status", "plans", ["status"]),
        index!("idx_plan_versions_org", "plan_versions", ["org_id"]),
        index!("idx_plan_versions_plan", "plan_versions", ["plan_id"]),
        index!(
            "idx_plan_versions_created_by",
            "plan_versions",
            ["created_by"]
        ),
        index!("idx_plan_reviews_org", "plan_reviews", ["org_id"]),
        index!(
            "idx_plan_reviews_version",
            "plan_reviews",
            ["plan_version_id"]
        ),
        index!("idx_plan_reviews_reviewer", "plan_reviews", ["reviewer_id"]),
    ]
}

/// Performs the foreign keys operation.
#[must_use]
pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "plans".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plans".to_owned(),
            from_column: "project_id".to_owned(),
            to_table: "projects".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plans".to_owned(),
            from_column: "created_by".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plan_versions".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plan_versions".to_owned(),
            from_column: "plan_id".to_owned(),
            to_table: "plans".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plan_versions".to_owned(),
            from_column: "created_by".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plan_reviews".to_owned(),
            from_column: "org_id".to_owned(),
            to_table: "organizations".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plan_reviews".to_owned(),
            from_column: "plan_version_id".to_owned(),
            to_table: "plan_versions".to_owned(),
            to_column: "id".to_owned(),
        },
        ForeignKeyDef {
            from_table: "plan_reviews".to_owned(),
            from_column: "reviewer_id".to_owned(),
            to_table: "users".to_owned(),
            to_column: "id".to_owned(),
        },
    ]
}

/// Performs the unique constraints operation.
#[must_use]
pub fn unique_constraints() -> Vec<super::UniqueConstraintDef> {
    vec![super::UniqueConstraintDef {
        table: "plan_versions".to_owned(),
        columns: vec!["plan_id".to_owned(), "version_number".to_owned()],
    }]
}
