//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
/// Constant value for `DEFAULT_MEMORY_LIMIT`.
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;
/// Constant value for `DEFAULT_CPU_LIMIT`.
pub const DEFAULT_CPU_LIMIT: usize = 4;
/// Constant value for `DEFAULT_DISK_IO_LIMIT`.
pub const DEFAULT_DISK_IO_LIMIT: u64 = 100 * 1024 * 1024;
/// Constant value for `DEFAULT_MAX_CONNECTIONS`.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 1000;
/// Constant value for `DEFAULT_MAX_REQUESTS_PER_CONNECTION`.
pub const DEFAULT_MAX_REQUESTS_PER_CONNECTION: u32 = 100;

/// Maximum depth of the GraphQL query tree.
pub const SCHEMA_DEPTH: usize = 15;

/// Maximum complexity score for GraphQL queries.
pub const SCHEMA_COMPLEXITY: usize = 250;

// ============================================================================
// MCP Handler Defaults
// ============================================================================

/// Default number of results for code and memory searches.
pub const DEFAULT_SEARCH_LIMIT: usize = crate::constants::values::DEFAULT_LIST_LIMIT;

/// Default number of memory items returned by list/timeline queries.
pub const DEFAULT_MEMORY_LIST_LIMIT: usize = crate::constants::values::DEFAULT_LIST_LIMIT;

/// Default number of sessions returned by session list queries.
pub const DEFAULT_SESSION_LIST_LIMIT: usize = crate::constants::values::DEFAULT_LIST_LIMIT;

/// Default number of results for VCS branch searches.
pub const DEFAULT_VCS_SEARCH_LIMIT: usize = 20;

/// Default maximum token budget for memory context injection.
pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 2000;

/// Estimated characters per token for size calculations.
pub const CHARS_PER_TOKEN_ESTIMATE: usize = 4;

/// Internal fetch multiplier applied to limit before filtering.
pub const MEMORY_FETCH_MULTIPLIER: usize = 5;

/// Default timeline depth (observations before/after anchor).
pub const DEFAULT_TIMELINE_DEPTH: usize = 5;

/// Maximum limit for observation list queries (pagination cap).
pub const OBSERVATION_LIST_MAX_LIMIT: usize = 1000;
