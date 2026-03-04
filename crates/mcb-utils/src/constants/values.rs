//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Domain layer constants definitions

// ============================================================================
// INDEXING DOMAIN CONSTANTS
// ============================================================================

/// Default batch size for indexing operations
pub const INDEXING_BATCH_SIZE: usize = 10;

/// Indexing status: started
pub const INDEXING_STATUS_STARTED: &str = "started";

/// Indexing status: completed
pub const INDEXING_STATUS_COMPLETED: &str = "completed";

/// Minimum character length for a code chunk to be indexed
pub const INDEXING_CHUNK_MIN_LENGTH: usize = 25;

/// Minimum number of lines for a code chunk to be indexed
pub const INDEXING_CHUNK_MIN_LINES: usize = 2;

/// Maximum number of chunks extracted from a single file
pub const INDEXING_CHUNKS_MAX_PER_FILE: usize = 50;

/// Maximum number of chunks returned for a single file when browsing vectors
pub const BROWSE_MAX_CHUNKS_PER_FILE: usize = 1000;

/// Default tombstone TTL for file hash cleanup (7 days in seconds).
pub const TOMBSTONE_TTL_SECS: u64 = 7 * 24 * 3600;

/// Default dashboard graph query limit.
pub const DEFAULT_DASHBOARD_LIMIT: usize = 30;

/// Default limit for session context search results.
pub const SESSION_SEARCH_LIMIT: usize = 10;

/// Milliseconds per second conversion factor.
pub const MS_PER_SEC: i64 = 1000;

// ============================================================================
// COMMON DEFAULTS
// ============================================================================

/// Default language identifier when language cannot be determined
pub const DEFAULT_LANGUAGE: &str = "unknown";

// ============================================================================
// PROVIDER DEFAULTS
// ============================================================================

/// Registry provider name for `SeaORM` database repositories.
pub const DEFAULT_DATABASE_PROVIDER: &str = "seaorm";

/// Registry provider name for universal language chunking.
pub const DEFAULT_LANGUAGE_PROVIDER: &str = "universal";

/// Registry provider name for Git VCS.
pub const DEFAULT_VCS_PROVIDER: &str = "git";

/// Default namespace for database repositories.
pub const DEFAULT_NAMESPACE: &str = "default";

/// Default limit for session queries to prevent unbounded result sets.
pub const DEFAULT_SESSION_LIMIT: u64 = 100;

/// Default limit for UI vector fetching.
pub const DEFAULT_BROWSE_LIMIT: usize = 50;
