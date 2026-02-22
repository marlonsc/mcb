//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "session_summaries",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text, nullable),
            crate::col!("project_id", Text),
            crate::col!("repo_id", Text, nullable),
            crate::col!("session_id", Text),
            crate::col!("topics", Text, nullable),
            crate::col!("decisions", Text, nullable),
            crate::col!("next_steps", Text, nullable),
            crate::col!("key_files", Text, nullable),
            crate::col!("origin_context", Text, nullable),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_summary_project", "session_summaries", ["project_id"]),
        crate::index!("idx_summary_session", "session_summaries", ["session_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!(
        "session_summaries",
        "project_id",
        "projects",
        "id"
    )]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
