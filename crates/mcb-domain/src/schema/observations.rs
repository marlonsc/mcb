//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{
    COL_OBSERVATION_TYPE, ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef,
};

pub fn table() -> TableDef {
    crate::table!(
        "observations",
        [
            crate::col!("id", Text, pk),
            crate::col!("project_id", Text),
            crate::col!("content", Text),
            crate::col!("content_hash", Text, unique),
            crate::col!("tags", Text, nullable),
            crate::col!(COL_OBSERVATION_TYPE, Text, nullable),
            crate::col!("metadata", Text, nullable),
            crate::col!("created_at", Integer),
            crate::col!("embedding_id", Text, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_obs_project", "observations", ["project_id"]),
        crate::index!("idx_obs_hash", "observations", ["content_hash"]),
        crate::index!("idx_obs_created", "observations", ["created_at"]),
        crate::index!("idx_obs_type", "observations", [COL_OBSERVATION_TYPE]),
        crate::index!("idx_obs_embedding", "observations", ["embedding_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!("observations", "project_id", "projects", "id")]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
