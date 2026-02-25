//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
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

/// Path fragment identifying a tests directory (used in single-pattern skip checks).
pub const TEST_DIR_FRAGMENT: &str = "/tests/";

/// Suffix identifying test source files.
pub const TEST_FILE_SUFFIX: &str = "_test.rs";

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

/// Lines to search forward for declarations after a marker.
pub const FORWARD_SEARCH_LINES: usize = 5;

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

/// Rust `async fn` keyword prefix.
pub const ASYNC_FN_PREFIX: &str = "async fn ";

/// All function declaration line prefixes (fn, pub fn, async fn, pub async fn).
pub const FN_PREFIXES: &[&str] = &[FN_PREFIX, PUB_FN_PREFIX, ASYNC_FN_PREFIX, PUB_ASYNC_FN_PREFIX];

/// Rust `mod ` keyword prefix.
pub const MOD_PREFIX: &str = "mod ";

// ============================================================================
// Control-Flow Detection (KISS / complexity)
// ============================================================================

/// Tokens that indicate control flow when contained in a line (with spaces).
pub const CONTROL_FLOW_CONTAINS_TOKENS: &[&str] = &[" if ", "} else", " match ", " else {"];

/// Tokens that indicate control flow when a line starts with them.
pub const CONTROL_FLOW_STARTS_WITH_TOKENS: &[&str] = &["if ", "match ", "for ", "while ", "loop "];

// ============================================================================
// Error Handling Detection Patterns
// ============================================================================

/// `.unwrap()` method call pattern.
pub const UNWRAP_CALL: &str = ".unwrap()";

/// `.expect(` method call pattern.
pub const EXPECT_CALL: &str = ".expect(";

// ============================================================================
// Validation Hint Patterns
// ============================================================================

/// Prefix for inline ignore-hint comments (`mcb-validate-ignore: `).
pub const VALIDATE_IGNORE_PREFIX: &str = "mcb-validate-ignore: ";

// ============================================================================
// DI / Implementation Suffix Patterns
// ============================================================================

/// Common concrete-type suffixes that indicate a DI violation.
pub const DI_IMPL_SUFFIXES: &[&str] = &["Impl", "Implementation", "Adapter"];

/// Handler file suffix (e.g. `foo_handler.rs`).
pub const HANDLER_FILE_SUFFIX: &str = "_handler.rs";

/// Repository file name suffix.
pub const REPOSITORY_FILE_SUFFIX: &str = "_repository";

/// Service file name suffix.
pub const SERVICE_FILE_SUFFIX: &str = "_service";

/// Factory file name suffix.
pub const FACTORY_FILE_SUFFIX: &str = "_factory";

// ============================================================================
// Error Module Detection
// ============================================================================

/// Error module file name.
pub const ERROR_MODULE_FILE: &str = "error.rs";

/// Error module name prefix.
pub const ERROR_FILE_PREFIX: &str = "error";

// ============================================================================
// Performance / Allocation Detection
// ============================================================================

/// Standard collection type prefixes for allocation detection in loops.
pub const HEAP_ALLOC_PREFIXES: &[&str] = &["Vec::", "String::", "HashMap::", "HashSet::"];

// ============================================================================
// Workspace Crate Prefixes
// ============================================================================

/// MCB workspace crate name prefix.
pub const MCB_CRATE_PREFIX: &str = "mcb-";

/// MCB dependency name prefix (without hyphen).
pub const MCB_DEPENDENCY_PREFIX: &str = "mcb";
