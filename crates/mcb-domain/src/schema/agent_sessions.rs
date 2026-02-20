//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "agent_sessions",
        [
            crate::col!("id", Text, pk),
            crate::col!("project_id", Text, nullable),
            crate::col!("worktree_id", Text, nullable),
            crate::col!("session_summary_id", Text),
            crate::col!("agent_type", Text),
            crate::col!("model", Text),
            crate::col!("parent_session_id", Text, nullable),
            crate::col!("started_at", Integer),
            crate::col!("ended_at", Integer, nullable),
            crate::col!("duration_ms", Integer, nullable),
            crate::col!("status", Text),
            crate::col!("prompt_summary", Text, nullable),
            crate::col!("result_summary", Text, nullable),
            crate::col!("token_count", Integer, nullable),
            crate::col!("tool_calls_count", Integer, nullable),
            crate::col!("delegations_count", Integer, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    crate::indexes_for_table!(
        "agent_sessions",
        {
            "idx_agent_sessions_summary" => ["session_summary_id"],
            "idx_agent_sessions_parent" => ["parent_session_id"],
            "idx_agent_sessions_type" => ["agent_type"],
            "idx_agent_sessions_project" => ["project_id"],
            "idx_agent_sessions_worktree" => ["worktree_id"],
            "idx_agent_sessions_started" => ["started_at"],
        }
    )
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!(
            "agent_sessions",
            "parent_session_id",
            "agent_sessions",
            "id"
        ),
        crate::fk!("agent_sessions", "project_id", "projects", "id"),
        crate::fk!("agent_sessions", "worktree_id", "worktrees", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
