//! Constants for refactoring validators.

/// Regex pattern for detecting type definitions (struct, trait, enum).
pub const TYPE_DEFINITION_REGEX: &str =
    r"(?:pub\s+)?(?:struct|trait|enum)\s+([A-Z][a-zA-Z0-9_]*)(?:\s*<|\s*\{|\s*;|\s*\(|\s+where)";

/// Path patterns for files to skip in duplicate detection.
pub const REFACTORING_SKIP_PATTERNS: &[&str] = &["/tests/", "_test.rs", ".archived", ".bak"];

/// Crate path delimiter for extracting crate names.
pub const CRATE_PATH_DELIMITER: &str = "/crates/";

/// Type name suffixes that suggest migration in progress.
pub const MIGRATION_TYPE_SUFFIXES: &[&str] = &[
    "Provider",
    "Processor",
    "Handler",
    "Service",
    "Repository",
    "Adapter",
    "Factory",
    "Publisher",
    "Subscriber",
];
