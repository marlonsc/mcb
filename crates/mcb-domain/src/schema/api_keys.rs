use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "api_keys",
        [
            crate::col!("id", Text, pk),
            crate::col!("user_id", Text),
            crate::col!("org_id", Text),
            crate::col!("key_hash", Text),
            crate::col!("name", Text),
            crate::col!("scopes_json", Text),
            crate::col!("expires_at", Integer, nullable),
            crate::col!("created_at", Integer),
            crate::col!("revoked_at", Integer, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_api_keys_user", "api_keys", ["user_id"]),
        crate::index!("idx_api_keys_org", "api_keys", ["org_id"]),
        crate::index!("idx_api_keys_key_hash", "api_keys", ["key_hash"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("api_keys", "user_id", "users", "id"),
        crate::fk!("api_keys", "org_id", "organizations", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
