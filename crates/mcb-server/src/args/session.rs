use mcb_domain::value_objects::ids::SessionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_enum! {
/// Actions available for session management operations
pub enum SessionAction {
    /// Create a new session.
    Create,
    /// Get an existing session.
    Get,
    /// Update an existing session.
    Update,
    /// List available sessions.
    List,
    /// Summarize a session.
    Summarize,
}
}

tool_schema! {
/// Arguments for session management tool operations
pub struct SessionArgs {
    /// Action: create, get, update, list, summarize.
    #[schemars(description = "Action: create, get, update, list, summarize")]
    pub action: SessionAction,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Session ID (required for get, update, summarize).
    #[schemars(
        description = "Session ID (required for get, update, summarize)",
        with = "SessionId"
    )]
    pub session_id: Option<SessionId>,

    /// Data payload for create/update (JSON object).
    #[schemars(
        description = "Data payload for create/update. create requires model and accepts session_summary_id?, agent_type? (or top-level args.agent_type), parent_session_id?, prompt_summary?, project_id?, worktree_id?; update accepts mutable session fields",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    /// Filter by project ID.
    #[schemars(description = "Filter by project ID", with = "String")]
    pub project_id: Option<String>,

    /// Filter by worktree ID.
    #[schemars(description = "Filter by worktree ID", with = "String")]
    pub worktree_id: Option<String>,

    /// Filter by parent session ID.
    #[schemars(description = "Filter by parent session ID", with = "String")]
    pub parent_session_id: Option<String>,

    /// Filter by agent type.
    #[schemars(description = "Filter by agent type", with = "String")]
    pub agent_type: Option<String>,

    /// Filter by status.
    #[schemars(description = "Filter by status", with = "String")]
    pub status: Option<String>,

    /// Maximum results for list.
    #[schemars(description = "Maximum results for list", with = "u32")]
    pub limit: Option<u32>,
}
}
