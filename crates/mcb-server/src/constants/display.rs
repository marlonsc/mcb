//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Display formatting and presentation constants.

// ============================================================================
// TRUNCATION
// ============================================================================

/// Default character length when truncating IDs in templates.
pub const DEFAULT_ID_TRUNCATE_LENGTH: usize = 8;

/// Default character length when truncating text in templates.
pub const DEFAULT_TEXT_TRUNCATE_LENGTH: usize = 80;

/// Maximum lines shown in a code preview snippet.
pub const CODE_PREVIEW_MAX_LINES: usize = 10;

// ============================================================================
// PERFORMANCE THRESHOLDS
// ============================================================================

/// Search duration threshold (milliseconds) for showing a slow-query warning.
pub const SEARCH_SLOW_THRESHOLD_MS: u128 = 1000;

// ============================================================================
// TIME CONSTANTS (for relative-time formatting)
// ============================================================================

/// Seconds in one minute.
pub const SECS_PER_MINUTE: i64 = 60;

/// Seconds in one hour.
pub const SECS_PER_HOUR: i64 = 3_600;

/// Seconds in one day.
pub const SECS_PER_DAY: i64 = 86_400;

/// Seconds in one week.
pub const SECS_PER_WEEK: i64 = 604_800;
