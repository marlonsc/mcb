use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
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
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_plan_versions_org", "plan_versions", ["org_id"]),
        crate::index!("idx_plan_versions_plan", "plan_versions", ["plan_id"]),
        crate::index!(
            "idx_plan_versions_created_by",
            "plan_versions",
            ["created_by"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("plan_versions", "org_id", "organizations", "id"),
        crate::fk!("plan_versions", "plan_id", "plans", "id"),
        crate::fk!("plan_versions", "created_by", "users", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!(
        "plan_versions",
        ["plan_id", "version_number"]
    )]
}
