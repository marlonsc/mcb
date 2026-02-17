use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "tool_calls",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("project_id", Text),
            crate::col!("repo_id", Text),
            crate::col!("session_id", Text),
            crate::col!("tool_name", Text),
            crate::col!("params_summary", Text, nullable),
            crate::col!("success", Integer),
            crate::col!("error_message", Text, nullable),
            crate::col!("duration_ms", Integer, nullable),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_tool_calls_session", "tool_calls", ["session_id"]),
        crate::index!("idx_tool_calls_tool", "tool_calls", ["tool_name"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!(
        "tool_calls",
        "session_id",
        "agent_sessions",
        "id"
    )]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
