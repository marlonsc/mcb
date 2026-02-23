//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
//! Constants for quality validators.

// ============================================================================
// UNWRAP DETECTION (unwrap.rs)
// ============================================================================

/// Safety justification comment markers.
pub const SAFETY_COMMENT_MARKERS: &[&str] = &["// SAFETY:", "// safety:"];

/// Ignore hint keywords for unwrap/expect suppression.
pub const IGNORE_HINT_KEYWORDS: &[&str] = &["lock_poisoning_recovery", "hardcoded_fallback"];

/// Number of lines before/after a detection to search for ignore hints.
pub const COMMENT_SEARCH_RADIUS: usize = 3;

/// Strings that indicate legitimate lock-poisoning `expect()` usage.
pub const LOCK_POISONING_STRINGS: &[&str] = &[
    "lock poisoned",
    "Lock poisoned",
    "poisoned",
    "Mutex poisoned",
];

// ============================================================================
// PANIC DETECTION (panic.rs)
// ============================================================================

/// Regex pattern for detecting `panic!()` macro usage.
pub const PANIC_REGEX: &str = r"panic!\s*\(";
