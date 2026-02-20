//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "teams",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("name", Text),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![crate::index!("idx_teams_org", "teams", ["org_id"])]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!("teams", "org_id", "organizations", "id")]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!("teams", ["org_id", "name"])]
}
