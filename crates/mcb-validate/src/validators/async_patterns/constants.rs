//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! Constants for async pattern validators.

/// Patterns for detecting wrong mutex types in async code: (pattern, description, suggestion).
pub const WRONG_MUTEX_PATTERNS: &[(&str, &str, &str)] = &[
    (
        r"use\s+std::sync::Mutex",
        "std::sync::Mutex import",
        "Use tokio::sync::Mutex for async code",
    ),
    (
        "std::sync::Mutex<",
        "std::sync::Mutex type",
        "Use tokio::sync::Mutex for async code",
    ),
    (
        r"use\s+std::sync::RwLock",
        "std::sync::RwLock import",
        "Use tokio::sync::RwLock for async code",
    ),
    (
        "std::sync::RwLock<",
        "std::sync::RwLock type",
        "Use tokio::sync::RwLock for async code",
    ),
];

/// Function name keywords that indicate intentional fire-and-forget spawns.
/// Includes constructor patterns that often spawn background workers.
pub const BACKGROUND_FN_PATTERNS: &[&str] = &[
    "spawn",
    "background",
    "graceful",
    "shutdown",
    "start",
    "run",
    "worker",
    "daemon",
    "listener",
    "handler",
    "process",
    "new",
    "with_",
    "init",
    "create",
    "build",
];
