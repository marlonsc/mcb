//! Agent session schema elements for the project schema.

use super::ForeignKeyDef;
use crate::constants::keys;
use crate::schema::memory::{IndexDef, TableDef};

pub fn tables() -> Vec<TableDef> {
    vec![
        crate::table!(
            "agent_sessions",
            [
                crate::col!(keys::ID, Text, pk),
                crate::col!(keys::SESSION_SUMMARY_ID, Text),
                crate::col!(keys::AGENT_TYPE, Text),
                crate::col!(keys::MODEL, Text),
                crate::col!(keys::PARENT_SESSION_ID, Text, nullable),
                crate::col!(keys::STARTED_AT, Integer),
                crate::col!(keys::ENDED_AT, Integer, nullable),
                crate::col!(keys::DURATION_MS, Integer, nullable),
                crate::col!(keys::STATUS, Text),
                crate::col!(keys::PROMPT_SUMMARY, Text, nullable),
                crate::col!(keys::RESULT_SUMMARY, Text, nullable),
                crate::col!(keys::TOKEN_COUNT, Integer, nullable),
                crate::col!(keys::TOOL_CALLS_COUNT, Integer, nullable),
                crate::col!(keys::DELEGATIONS_COUNT, Integer, nullable),
            ]
        ),
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
        ),
        crate::table!(
            "tool_calls",
            [
                crate::col!("id", Text, pk),
                crate::col!("session_id", Text),
                crate::col!("tool_name", Text),
                crate::col!("params_summary", Text, nullable),
                crate::col!("success", Integer),
                crate::col!("error_message", Text, nullable),
                crate::col!("duration_ms", Integer, nullable),
                crate::col!("created_at", Integer),
            ]
        ),
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
        ),
    ]
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_agent_sessions_parent",
            "agent_sessions",
            ["parent_session_id"]
        ),
        crate::index!("idx_agent_sessions_type", "agent_sessions", ["agent_type"]),
        crate::index!(
            "idx_agent_sessions_started",
            "agent_sessions",
            ["started_at"]
        ),
        crate::index!(
            "idx_delegations_parent",
            "delegations",
            ["parent_session_id"]
        ),
        crate::index!("idx_delegations_child", "delegations", ["child_session_id"]),
        crate::index!("idx_tool_calls_session", "tool_calls", ["session_id"]),
        crate::index!("idx_tool_calls_tool", "tool_calls", ["tool_name"]),
        crate::index!("idx_checkpoints_session", "checkpoints", ["session_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "agent_sessions".to_string(),
            from_column: "session_summary_id".to_string(),
            to_table: "session_summaries".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "agent_sessions".to_string(),
            from_column: "parent_session_id".to_string(),
            to_table: "agent_sessions".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "delegations".to_string(),
            from_column: "parent_session_id".to_string(),
            to_table: "agent_sessions".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "delegations".to_string(),
            from_column: "child_session_id".to_string(),
            to_table: "agent_sessions".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "tool_calls".to_string(),
            from_column: "session_id".to_string(),
            to_table: "agent_sessions".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "checkpoints".to_string(),
            from_column: "session_id".to_string(),
            to_table: "agent_sessions".to_string(),
            to_column: "id".to_string(),
        },
    ]
}
