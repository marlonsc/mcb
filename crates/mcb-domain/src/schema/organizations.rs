//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "organizations",
        [
            crate::col!("id", Text, pk),
            crate::col!("name", Text),
            crate::col!("slug", Text, unique),
            crate::col!("settings_json", Text),
            crate::col!("created_at", Integer),
            crate::col!("updated_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![crate::index!(
        "idx_organizations_name",
        "organizations",
        ["name"]
    )]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    Vec::new()
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
