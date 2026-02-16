//! Validation allow-lists and skip patterns.
//!
//! Types, names, files, and directory patterns that are intentionally
//! excluded from certain validation rules.

/// Utility types that are intentionally duplicated to avoid cross-crate dependencies.
pub const UTILITY_TYPES: &[&str] = &["HttpResponseUtils", "CacheStats", "TimedOperation"];

/// Generic type names expected to appear in multiple places.
pub const GENERIC_TYPE_NAMES: &[&str] = &[
    "Error",
    "Result",
    "Config",
    "Builder",
    "Context",
    "State",
    "Options",
    "Params",
    "Settings",
    "Message",
    "Request",
    "Response",
    "CacheConfig",
];

/// File names to skip for test completeness checks.
pub const REFACTORING_SKIP_FILES: &[&str] = &[
    "mod",
    "lib",
    "main",
    "constants",
    "thresholds",
    "error",
    "types",
];

/// Directory patterns to skip for test completeness checks (tested via integration).
pub const REFACTORING_SKIP_DIR_PATTERNS: &[&str] = &["/di/", "/config/", "/models/", "/ports/"];
