//! Default limits and thresholds for MCP tool handlers and admin endpoints.

// ============================================================================
// SEARCH & LIST DEFAULTS
// ============================================================================

/// Default number of results for code and memory searches.
pub const DEFAULT_SEARCH_LIMIT: usize = 10;

/// Default number of memory items returned by list/timeline queries.
pub const DEFAULT_MEMORY_LIMIT: usize = 10;

/// Default number of sessions returned by session list queries.
pub const DEFAULT_SESSION_LIST_LIMIT: usize = 10;

/// Default number of results for VCS branch searches.
pub const DEFAULT_VCS_SEARCH_LIMIT: usize = 20;

/// Default file list limit for admin browse endpoints.
pub const DEFAULT_BROWSE_FILES_LIMIT: usize = 100;

/// Default list-of-values (LOV) limit for admin dropdown endpoints.
pub const DEFAULT_LOV_LIMIT: usize = 50;

// ============================================================================
// MEMORY INJECTION
// ============================================================================

/// Default maximum token budget for memory context injection.
pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 2000;

/// Estimated characters per token for size calculations.
pub const CHARS_PER_TOKEN_ESTIMATE: usize = 4;

/// Internal fetch multiplier applied to limit before filtering.
pub const MEMORY_FETCH_MULTIPLIER: usize = 5;

/// Default timeline depth (observations before/after anchor).
pub const DEFAULT_TIMELINE_DEPTH: usize = 5;

// ============================================================================
// SERVER LIFECYCLE
// ============================================================================

/// Default graceful shutdown timeout in seconds.
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u64 = 30;
