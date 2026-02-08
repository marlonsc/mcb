//! Schema constants
//!
//! Contains column names and other schema-related constants that are used
//! across the domain, infrastructure, and server layers.

/// Column/Field name for "delegations_count"
pub const DELEGATIONS_COUNT: &str = "delegations_count";

/// Column/Field name for "tool_calls_count"
pub const TOOL_CALLS_COUNT: &str = "tool_calls_count";

/// Column/Field name for "token_count"
pub const TOKEN_COUNT: &str = "token_count";

/// Column/Field name for "duration_ms"
pub const DURATION_MS: &str = "duration_ms";

/// Column/Field name for "id"
pub const ID: &str = "id";

/// Column/Field name for "name"
pub const NAME: &str = "name";

/// Column/Field name for "created_at"
pub const CREATED_AT: &str = "created_at";

/// Column/Field name for "updated_at"
pub const UPDATED_AT: &str = "updated_at";

/// Column/Field name for "status"
pub const STATUS: &str = "status";

/// Column/Field name for "model"
pub const MODEL: &str = "model";

/// Column/Field name for "agent_type"
pub const AGENT_TYPE: &str = "agent_type";

/// Column/Field name for "parent_session_id"
pub const PARENT_SESSION_ID: &str = "parent_session_id";

/// Column/Field name for "child_session_id"
pub const CHILD_SESSION_ID: &str = "child_session_id";

/// Column/Field name for "started_at"
pub const STARTED_AT: &str = "started_at";

/// Column/Field name for "ended_at"
pub const ENDED_AT: &str = "ended_at";

/// Column/Field name for "prompt_summary"
pub const PROMPT_SUMMARY: &str = "prompt_summary";

/// Column/Field name for "result_summary"
pub const RESULT_SUMMARY: &str = "result_summary";

/// Column/Field name for "session_summary_id"
pub const SESSION_SUMMARY_ID: &str = "session_summary_id";
