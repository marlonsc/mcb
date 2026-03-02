//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Constants for performance validators (Arc/Mutex, iterators, strings, loop allocations).

use crate::constants::common::CONTEXT_PREVIEW_LENGTH;

// ============================================================================
// ARC/MUTEX OVERUSE (pattern_checks.rs)
// ============================================================================

/// Patterns for detecting Arc/Mutex overuse: (pattern, description, suggestion).
pub const ARC_MUTEX_OVERUSE_PATTERNS: &[(&str, &str, &str)] = &[
    ("Arc<Arc<", "Nested Arc<Arc<>>", "Use single Arc instead"),
    ("Mutex<bool>", "Mutex<bool>", "Use AtomicBool instead"),
    ("Mutex<usize>", "Mutex<usize>", "Use AtomicUsize instead"),
    ("Mutex<u32>", "Mutex<u32>", "Use AtomicU32 instead"),
    ("Mutex<u64>", "Mutex<u64>", "Use AtomicU64 instead"),
    ("Mutex<i32>", "Mutex<i32>", "Use AtomicI32 instead"),
    ("Mutex<i64>", "Mutex<i64>", "Use AtomicI64 instead"),
    ("RwLock<bool>", "RwLock<bool>", "Use AtomicBool instead"),
];

// ============================================================================
// INEFFICIENT ITERATORS (pattern_checks.rs)
// ============================================================================

/// Patterns for detecting inefficient iterator usage.
pub const INEFFICIENT_ITERATOR_PATTERNS: &[(&str, &str, &str)] = &[
    (
        r"\.iter\(\)\.cloned\(\)\.take\(",
        ".iter().cloned().take()",
        "Use .iter().take().cloned() instead",
    ),
    (
        r"\.iter\(\)\.cloned\(\)\.last\(",
        ".iter().cloned().last()",
        "Use .iter().last().cloned() instead",
    ),
    (
        r#"\.collect::<Vec<String>>\(\)\.join\(\s*""\s*\)"#,
        r#".collect::<Vec<String>>().join("")"#,
        "Use .collect::<String>() instead",
    ),
    (
        r"\.repeat\(1\)",
        ".repeat(1)",
        "Use .clone() instead of .repeat(1)",
    ),
];

// ============================================================================
// INEFFICIENT STRINGS (pattern_checks.rs)
// ============================================================================

/// Patterns for detecting inefficient string handling.
pub const INEFFICIENT_STRING_PATTERNS: &[(&str, &str, &str)] = &[
    (
        r#"format!\s*\(\s*"\{\}"\s*,\s*\w+\s*\)"#,
        "format!(\"{}\", var)",
        "Use var.to_string() or &var instead",
    ),
    (
        r"\.to_string\(\)\.to_string\(\)",
        ".to_string().to_string()",
        "Remove redundant .to_string()",
    ),
    (
        r"\.to_owned\(\)\.to_owned\(\)",
        ".to_owned().to_owned()",
        "Remove redundant .to_owned()",
    ),
];

// ============================================================================
// LOOP CHECKS (loop_checks.rs)
// ============================================================================

/// Regex pattern for detecting `.clone()` calls.
pub const CLONE_REGEX: &str = r"\.clone\(\)";

/// Regex patterns for detecting allocations in loops.
pub const LOOP_ALLOCATION_PATTERNS: &[&str] = &[
    r"Vec::new\(\)",
    r"Vec::with_capacity\(",
    r"String::new\(\)",
    r"String::with_capacity\(",
    r"HashMap::new\(\)",
    r"HashSet::new\(\)",
];

/// Maximum characters of context to include in clone-in-loop violations.
pub const CONTEXT_TRUNCATION_LENGTH: usize = CONTEXT_PREVIEW_LENGTH;
