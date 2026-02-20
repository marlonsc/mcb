//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "branches",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("project_id", Text, nullable),
            crate::col!("repository_id", Text),
            crate::col!("name", Text),
            crate::col!("is_default", Integer),
            crate::col!("head_commit", Text),
            crate::col!("upstream", Text, nullable),
            crate::col!("origin_context", Text, nullable),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![crate::index!(
        "idx_branches_repo",
        "branches",
        ["repository_id"]
    )]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!(
        "branches",
        "repository_id",
        "repositories",
        "id"
    )]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!("branches", ["repository_id", "name"])]
}
