//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
/// File hash tombstone TTL in seconds (30 days)
pub const FILE_HASH_TOMBSTONE_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Max characters for SQL statement preview in log messages.
pub const SQL_PREVIEW_CHAR_LIMIT: usize = 120;

// ============================================================================
// Observation / query limits
// ============================================================================

/// Maximum limit for observation list queries (pagination cap).
pub const OBSERVATION_LIST_MAX_LIMIT: usize = 1000;
