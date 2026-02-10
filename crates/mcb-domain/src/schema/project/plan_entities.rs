use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

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
                crate::col!("plan_version_id", Text),
                crate::col!("reviewer_id", Text),
                crate::col!("verdict", Text),
                crate::col!("feedback", Text),
                crate::col!("created_at", Integer),
            ]
        ),
    ]
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        index!("idx_plans_org", "plans", ["org_id"]),
        index!("idx_plans_project", "plans", ["project_id"]),
        index!("idx_plans_status", "plans", ["status"]),
        index!("idx_plan_versions_plan", "plan_versions", ["plan_id"]),
        index!(
            "idx_plan_reviews_version",
            "plan_reviews",
            ["plan_version_id"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "plans".to_string(),
            from_column: "org_id".to_string(),
            to_table: "organizations".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "plans".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "plans".to_string(),
            from_column: "created_by".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "plan_versions".to_string(),
            from_column: "plan_id".to_string(),
            to_table: "plans".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "plan_versions".to_string(),
            from_column: "created_by".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "plan_reviews".to_string(),
            from_column: "plan_version_id".to_string(),
            to_table: "plan_versions".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "plan_reviews".to_string(),
            from_column: "reviewer_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        },
    ]
}

pub fn unique_constraints() -> Vec<super::UniqueConstraintDef> {
    vec![super::UniqueConstraintDef {
        table: "plan_versions".to_string(),
        columns: vec!["plan_id".to_string(), "version_number".to_string()],
    }]
}
