//! Common validation constants shared across multiple validators.
//!
//! Cross-cutting code patterns: comment prefixes, test markers,
//! declaration prefixes, and standard skip lists.

/// Marker for test module configuration attribute.
pub const CFG_TEST_MARKER: &str = "#[cfg(test)]";

/// Line comment prefix.
pub const COMMENT_PREFIX: &str = "//";

/// Doc comment prefix.
pub const DOC_COMMENT_PREFIX: &str = "///";

/// Module-level doc comment prefix.
pub const MODULE_DOC_PREFIX: &str = "//!";

/// Attribute macro prefix.
pub const ATTRIBUTE_PREFIX: &str = "#[";

/// Line prefixes that indicate a const or static declaration.
pub const CONST_DECLARATION_PREFIXES: &[&str] = &["const ", "pub const ", "static ", "pub static "];

/// Path fragments that indicate a test file or directory.
pub const TEST_PATH_PATTERNS: &[&str] = &["/tests/", "/target/", "_test.rs", "test.rs"];

/// File stems that should be skipped in many validators.
pub const STANDARD_SKIP_FILES: &[&str] = &["lib", "mod", "main", "build"];

/// File name keywords that identify a constants file (skip in magic number checks, etc.).
pub const CONSTANTS_FILE_KEYWORDS: &[&str] = &["constant", "config"];

/// Prefix for test function names.
pub const TEST_FUNCTION_PREFIX: &str = "test_";

// ============================================================================
// Preview / Truncation Lengths
// ============================================================================

/// Short context preview (match expressions, error patterns) — 60 chars.
pub const SHORT_PREVIEW_LENGTH: usize = 60;

/// Standard context preview (async patterns, spawn context) — 80 chars.
pub const CONTEXT_PREVIEW_LENGTH: usize = 80;

// ============================================================================
// Search Radius Constants
// ============================================================================

/// Lines to search backward for enclosing function names.
pub const FUNCTION_NAME_SEARCH_LINES: usize = 10;

/// Lines to search backward for async trait attributes.
pub const ATTR_SEARCH_LINES: usize = 5;

/// Max forward offset for balanced-brace block extraction in `scan.rs`.
pub const MAX_BLOCK_SEARCH_OFFSET: usize = 20;

// ============================================================================
// Rust Code Pattern Strings
// ============================================================================

/// Rust `fn` keyword prefix.
pub const FN_PREFIX: &str = "fn ";

/// Rust `pub fn` prefix.
pub const PUB_FN_PREFIX: &str = "pub fn ";

/// Rust `pub async fn` prefix.
pub const PUB_ASYNC_FN_PREFIX: &str = "pub async fn ";

/// Rust `let` binding prefix.
pub const LET_PREFIX: &str = "let ";

/// Rust `pub use` re-export prefix.
pub const PUB_USE_PREFIX: &str = "pub use";

/// Rust `use` import prefix.
pub const USE_PREFIX: &str = "use ";
