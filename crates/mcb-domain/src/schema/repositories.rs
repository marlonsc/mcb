//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "repositories",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("project_id", Text),
            crate::col!("name", Text),
            crate::col!("url", Text),
            crate::col!("local_path", Text),
            crate::col!("vcs_type", Text),
            crate::col!("origin_context", Text, nullable),
            crate::col!("created_at", Integer),
            crate::col!("updated_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_repositories_org", "repositories", ["org_id"]),
        crate::index!("idx_repositories_project", "repositories", ["project_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("repositories", "org_id", "organizations", "id"),
        crate::fk!("repositories", "project_id", "projects", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!(
        "repositories",
        ["org_id", "project_id", "name"]
    )]
}
