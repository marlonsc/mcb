//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use mcb_domain::value_objects::ids::SessionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::args::schema_helpers::ObjectDataSchema;

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
        with = "ObjectDataSchema"
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

// ---------------------------------------------------------------------------
// MCP-facing single-purpose tools
// ---------------------------------------------------------------------------

tool_action! {
    /// Arguments for the `store_memory` tool.
    pub struct StoreMemoryArgs => MemoryArgs {
        #[schemars(description = "Content: plain text or JSON {content, type?, tags?, metadata?}", with = "ObjectDataSchema")]
        data: Option<serde_json::Value>,
        #[schemars(description = "Tags for categorization", with = "Vec<String>")]
        tags: Option<Vec<String>>
        ;
        hidden {
            org_id: Option<String>, project_id: Option<String>, repo_id: Option<String>,
            session_id: Option<SessionId>, parent_session_id: Option<String>,
        }
        ;
        convert |a| {
            action: MemoryAction::Store, resource: MemoryResource::Observation,
            data: a.data, ids: None, tags: a.tags, query: None,
            anchor_id: None, depth_before: None, depth_after: None,
            window_secs: None, observation_types: None, max_tokens: None, limit: None,
        }
    }
}

tool_action! {
    /// Arguments for the `get_memories` tool.
    pub struct GetMemoriesArgs => MemoryArgs {
        #[schemars(description = "Memory IDs to retrieve", with = "Vec<String>")]
        ids: Option<Vec<String>>
        ;
        hidden {
            org_id: Option<String>, project_id: Option<String>, repo_id: Option<String>,
            session_id: Option<SessionId>, parent_session_id: Option<String>,
        }
        ;
        convert |a| {
            action: MemoryAction::Get, resource: MemoryResource::Observation,
            data: None, ids: a.ids, tags: None, query: None,
            anchor_id: None, depth_before: None, depth_after: None,
            window_secs: None, observation_types: None, max_tokens: None, limit: None,
        }
    }
}

tool_action! {
    /// Arguments for the `list_memories` tool.
    pub struct ListMemoriesArgs => MemoryArgs {
        #[schemars(description = "Search query to filter", with = "String")]
        query: Option<String>,
        #[schemars(description = "Filter by tags", with = "Vec<String>")]
        tags: Option<Vec<String>>,
        #[schemars(description = "Maximum results", with = "u32")]
        limit: Option<u32>,
        #[schemars(description = "Time window in seconds", with = "i64")]
        window_secs: Option<i64>
        ;
        hidden {
            org_id: Option<String>, project_id: Option<String>, repo_id: Option<String>,
            session_id: Option<SessionId>, parent_session_id: Option<String>,
        }
        ;
        convert |a| {
            action: MemoryAction::List, resource: MemoryResource::Observation,
            data: None, ids: None, tags: a.tags, query: a.query,
            anchor_id: None, depth_before: None, depth_after: None,
            window_secs: a.window_secs, observation_types: None, max_tokens: None, limit: a.limit,
        }
    }
}

tool_action! {
    /// Arguments for the `memory_timeline` tool.
    pub struct MemoryTimelineArgs => MemoryArgs {
        #[schemars(description = "Anchor observation ID to center timeline on")]
        anchor_id: String,
        #[schemars(description = "Items before anchor (default: 5)", with = "usize")]
        depth_before: Option<usize>,
        #[schemars(description = "Items after anchor (default: 5)", with = "usize")]
        depth_after: Option<usize>
        ;
        hidden {
            org_id: Option<String>, project_id: Option<String>, repo_id: Option<String>,
            session_id: Option<SessionId>, parent_session_id: Option<String>,
        }
        ;
        convert |a| {
            action: MemoryAction::Timeline, resource: MemoryResource::Observation,
            data: None, ids: None, tags: None, query: None,
            anchor_id: Some(a.anchor_id), depth_before: a.depth_before, depth_after: a.depth_after,
            window_secs: None, observation_types: None, max_tokens: None, limit: None,
        }
    }
}

tool_action! {
    /// Arguments for the `inject_context` tool.
    pub struct InjectContextArgs => MemoryArgs {
        #[schemars(description = "Observation types to include", with = "Vec<String>")]
        observation_types: Option<Vec<String>>,
        #[schemars(description = "Maximum token budget", with = "usize")]
        max_tokens: Option<usize>
        ;
        hidden {
            org_id: Option<String>, project_id: Option<String>, repo_id: Option<String>,
            session_id: Option<SessionId>, parent_session_id: Option<String>,
        }
        ;
        convert |a| {
            action: MemoryAction::Inject, resource: MemoryResource::Observation,
            data: None, ids: None, tags: None, query: None,
            anchor_id: None, depth_before: None, depth_after: None,
            window_secs: None, observation_types: a.observation_types, max_tokens: a.max_tokens, limit: None,
        }
    }
}
