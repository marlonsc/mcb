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
