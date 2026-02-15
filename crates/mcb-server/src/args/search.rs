use mcb_domain::value_objects::ids::SessionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_enum! {
/// Resources available for semantic search.
pub enum SearchResource {
    /// Search across the indexed codebase.
    Code,
    /// Search across the memory (observations).
    Memory,
    /// Search across context snapshots.
    Context,
}
}

tool_schema! {
/// Arguments for the search tool.
pub struct SearchArgs {
    /// Natural language search query.
    #[schemars(description = "Natural language search query")]
    #[validate(length(min = 1))]
    pub query: String,

    /// Resource to search: code or memory.
    #[schemars(description = "Resource to search: code or memory")]
    pub resource: SearchResource,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Collection name.
    #[schemars(description = "Collection name", with = "String")]
    pub collection: Option<String>,

    /// File extensions to include (code search only).
    #[schemars(
        description = "File extensions to include (code search only)",
        with = "Vec<String>"
    )]
    pub extensions: Option<Vec<String>>,

    /// Additional search filters.
    #[schemars(description = "Additional search filters", with = "Vec<String>")]
    pub filters: Option<Vec<String>>,

    /// Maximum results to return.
    #[schemars(description = "Maximum results to return", with = "u32")]
    pub limit: Option<u32>,

    /// Minimum similarity score (0.0-1.0).
    #[schemars(description = "Minimum similarity score (0.0-1.0)", with = "f32")]
    #[validate(range(min = 0.0, max = 1.0, message = "Min score must be 0.0-1.0"))]
    pub min_score: Option<f32>,

    /// Filter by tags (for memory search).
    #[schemars(
        description = "Filter by tags (for memory search)",
        with = "Vec<String>"
    )]
    pub tags: Option<Vec<String>>,

    /// Filter by session ID (for memory search).
    #[schemars(
        description = "Filter by session ID (for memory search)",
        with = "SessionId"
    )]
    pub session_id: Option<SessionId>,

    /// JWT token for authenticated requests.
    #[schemars(description = "JWT token for authenticated requests", with = "String")]
    pub token: Option<String>,
}
}
