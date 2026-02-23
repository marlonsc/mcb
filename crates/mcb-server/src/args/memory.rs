//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use mcb_domain::value_objects::ids::SessionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_enum! {
/// Actions available for the memory tool.
pub enum MemoryAction {
    /// Store a new memory item.
    Store,
    /// Get a specific memory item by ID.
    Get,
    /// List memory items with filters.
    List,
    /// Get a timeline of memory items.
    Timeline,
    /// Inject relevant memory items into context.
    Inject,
}
}

tool_enum! {
/// Resource types for the memory tool.
pub enum MemoryResource {
    /// General observation.
    Observation,
    /// Tool execution result.
    Execution,
    /// Architectural quality gate.
    QualityGate,
    /// Common error pattern.
    ErrorPattern,
    /// Session metadata.
    Session,
}
}

tool_schema! {
/// Arguments for the memory tool.
pub struct MemoryArgs {
    /// Action: store, get, list, timeline, inject.
    #[schemars(description = "Action: store, get, list, timeline, inject")]
    pub action: MemoryAction,

    /// Resource type: observation, execution, `quality_gate`, `error_pattern`, session.
    #[schemars(
        description = "Resource type: observation, execution, quality_gate, error_pattern, session"
    )]
    pub resource: MemoryResource,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Data payload for store actions (JSON object).
    #[schemars(
        description = "Data payload for store action. observation: {content, type?, tags?, metadata?}; execution: {command, output?, status?}; quality_gate: {gate_name, status, details?}; error_pattern: {error_type, message, fix?}; session: {session_id, topics?, decisions?, next_steps?, key_files?}",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    /// Resource IDs for get action.
    #[schemars(description = "Resource IDs for get action", with = "Vec<String>")]
    pub ids: Option<Vec<String>>,

    /// Filter by project ID.
    #[schemars(description = "Filter by project ID", with = "String")]
    pub project_id: Option<String>,

    /// Filter by repository ID.
    #[schemars(description = "Filter by repository ID", with = "String")]
    pub repo_id: Option<String>,

    /// Filter by session ID.
    #[schemars(description = "Filter by session ID", with = "SessionId")]
    pub session_id: Option<SessionId>,

    /// Filter by parent session ID.
    #[schemars(description = "Filter by parent session ID", with = "String")]
    pub parent_session_id: Option<String>,

    /// Filter by tags.
    #[schemars(description = "Filter by tags", with = "Vec<String>")]
    pub tags: Option<Vec<String>>,

    /// Query string for list/search actions.
    #[schemars(description = "Query string for list/search actions", with = "String")]
    pub query: Option<String>,

    /// Anchor observation ID (for timeline action).
    #[schemars(
        description = "Anchor observation ID (for timeline action)",
        with = "String"
    )]
    pub anchor_id: Option<String>,

    /// Timeline depth before the anchor (default: 5).
    #[schemars(
        description = "Timeline depth before the anchor (default: 5)",
        with = "usize"
    )]
    pub depth_before: Option<usize>,

    /// Timeline depth after the anchor (default: 5).
    #[schemars(
        description = "Timeline depth after the anchor (default: 5)",
        with = "usize"
    )]
    pub depth_after: Option<usize>,

    /// Time window in seconds (for timeline action).
    #[schemars(
        description = "Time window in seconds (for timeline action)",
        with = "i64"
    )]
    pub window_secs: Option<i64>,

    /// Observation types to include (inject action).
    #[schemars(
        description = "Observation types to include (inject action)",
        with = "Vec<String>"
    )]
    pub observation_types: Option<Vec<String>>,

    /// Maximum token budget for injected context.
    #[schemars(
        description = "Maximum token budget for injected context",
        with = "usize"
    )]
    pub max_tokens: Option<usize>,

    /// Maximum results.
    #[schemars(description = "Maximum results", with = "u32")]
    pub limit: Option<u32>,
}
}
