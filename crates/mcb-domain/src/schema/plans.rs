use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
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
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_plans_org", "plans", ["org_id"]),
        crate::index!("idx_plans_project", "plans", ["project_id"]),
        crate::index!("idx_plans_status", "plans", ["status"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("plans", "org_id", "organizations", "id"),
        crate::fk!("plans", "project_id", "projects", "id"),
        crate::fk!("plans", "created_by", "users", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
