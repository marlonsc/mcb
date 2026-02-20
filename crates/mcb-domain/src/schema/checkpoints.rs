use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "checkpoints",
        [
            crate::col!("id", Text, pk),
            crate::col!("session_id", Text),
            crate::col!("checkpoint_type", Text),
            crate::col!("description", Text),
            crate::col!("snapshot_data", Text),
            crate::col!("created_at", Integer),
            crate::col!("restored_at", Integer, nullable),
            crate::col!("expired", Integer, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![crate::index!(
        "idx_checkpoints_session",
        "checkpoints",
        ["session_id"]
    )]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!(
        "checkpoints",
        "session_id",
        "agent_sessions",
        "id"
    )]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
