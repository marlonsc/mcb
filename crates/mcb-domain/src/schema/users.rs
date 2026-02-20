//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "users",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("email", Text),
            crate::col!("display_name", Text),
            crate::col!("role", Text),
            crate::col!("api_key_hash", Text, nullable),
            crate::col!("created_at", Integer),
            crate::col!("updated_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_users_org", "users", ["org_id"]),
        crate::index!("idx_users_email", "users", ["email"]),
        crate::index!("idx_users_api_key_hash", "users", ["api_key_hash"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!("users", "org_id", "organizations", "id")]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!("users", ["org_id", "email"])]
}
