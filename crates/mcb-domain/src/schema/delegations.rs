use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "delegations",
        [
            crate::col!("id", Text, pk),
            crate::col!("parent_session_id", Text),
            crate::col!("child_session_id", Text),
            crate::col!("prompt", Text),
            crate::col!("prompt_embedding_id", Text, nullable),
            crate::col!("result", Text, nullable),
            crate::col!("success", Integer),
            crate::col!("created_at", Integer),
            crate::col!("completed_at", Integer, nullable),
            crate::col!("duration_ms", Integer, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_delegations_parent",
            "delegations",
            ["parent_session_id"]
        ),
        crate::index!("idx_delegations_child", "delegations", ["child_session_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("delegations", "parent_session_id", "agent_sessions", "id"),
        crate::fk!("delegations", "child_session_id", "agent_sessions", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
