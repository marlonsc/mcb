//! Default values for validation settings.

/// Default cyclomatic complexity threshold.
pub const DEFAULT_COMPLEXITY_THRESHOLD: u32 = 15;

/// Default TDG score threshold (0-100, higher is worse).
pub const DEFAULT_TDG_THRESHOLD: u32 = 50;

/// Default max lines per file before triggering a size violation.
pub const DEFAULT_MAX_FILE_LINES: usize = 500;
