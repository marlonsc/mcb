//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Schema constants
//!
//! Contains column names and other schema-related constants that are used
//! across the domain, infrastructure, and server layers.

// ============================================================================
// Column / Field Names (macro-generated)
// ============================================================================

define_str_consts! {
    /// Column/Field name for "`delegations_count`".
    DELEGATIONS_COUNT = "delegations_count";
    /// Column/Field name for "`tool_calls_count`".
    TOOL_CALLS_COUNT = "tool_calls_count";
    /// Column/Field name for "`token_count`".
    TOKEN_COUNT = "token_count";
    /// Column/Field name for "`duration_ms`".
    DURATION_MS = "duration_ms";
    /// Column/Field name for "id".
    ID = "id";
    /// Column/Field name for "name".
    NAME = "name";
    /// Column/Field name for "`created_at`".
    CREATED_AT = "created_at";
    /// Column/Field name for "status".
    STATUS = "status";
    /// Column/Field name for "model".
    MODEL = "model";
    /// Column/Field name for "`agent_type`".
    AGENT_TYPE = "agent_type";
    /// Column/Field name for "`parent_session_id`".
    PARENT_SESSION_ID = "parent_session_id";
    /// Column/Field name for "`started_at`".
    STARTED_AT = "started_at";
    /// Column/Field name for "`ended_at`".
    ENDED_AT = "ended_at";
    /// Column/Field name for "`prompt_summary`".
    PROMPT_SUMMARY = "prompt_summary";
    /// Column/Field name for "`result_summary`".
    RESULT_SUMMARY = "result_summary";
    /// Column/Field name for "`session_summary_id`".
    SESSION_SUMMARY_ID = "session_summary_id";
    /// Project ID column key.
    PROJECT_ID = "project_id";
    /// Worktree ID column key.
    WORKTREE_ID = "worktree_id";
    /// Repository ID column key.
    REPO_ID = "repo_id";
    /// Repository path column key.
    REPO_PATH = "repo_path";
    /// Column/Field name for "`org_id`".
    ORG_ID = "org_id";
}

// ============================================================================
// Metadata Keys (macro-generated)
// ============================================================================

define_str_consts! {
    /// Metadata key for "`start_line`".
    METADATA_KEY_START_LINE = "start_line";
    /// Metadata key for "`end_line`".
    METADATA_KEY_END_LINE = "end_line";
    /// Metadata key for "`chunk_type`".
    METADATA_KEY_CHUNK_TYPE = "chunk_type";
    /// Metadata key for "`file_path`".
    METADATA_KEY_FILE_PATH = "file_path";
    /// Metadata key for "`vectors_count`".
    METADATA_KEY_VECTORS_COUNT = "vectors_count";
    /// Metadata key for "content".
    METADATA_KEY_CONTENT = "content";
    /// Metadata key for "type".
    METADATA_KEY_TYPE = "type";
    /// Metadata key for "tags".
    METADATA_KEY_TAGS = "tags";
    /// Metadata key for "language".
    METADATA_KEY_LANGUAGE = "language";
    /// Metadata key for "`session_id`".
    METADATA_KEY_SESSION_ID = "session_id";
    /// Metadata key for "`line_number`".
    METADATA_KEY_LINE_NUMBER = "line_number";
}

// ============================================================================
// JSON Response Field Names (macro-generated)
// ============================================================================

define_str_consts! {
    /// JSON field: observation ID.
    FIELD_OBSERVATION_ID = "observation_id";
    /// JSON field: observation type.
    FIELD_OBSERVATION_TYPE = "observation_type";
    /// JSON field: message text.
    FIELD_MESSAGE = "message";
    /// Response count field name.
    FIELD_COUNT = "count";
    /// Response results array field name.
    FIELD_RESULTS = "results";
    /// Search query echo field name.
    FIELD_QUERY = "query";
    /// Updated flag field name.
    FIELD_UPDATED = "updated";
    /// Branch name field name.
    FIELD_BRANCH = "branch";
    /// Commit reference field name.
    FIELD_COMMIT = "commit";
    /// Sessions list field name.
    FIELD_SESSIONS = "sessions";
    /// Observations list field name.
    FIELD_OBSERVATIONS = "observations";
}
