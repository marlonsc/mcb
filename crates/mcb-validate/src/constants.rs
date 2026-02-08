//! Validation constants
//!
//! Centralized constants for architecture validation, refactoring checks,
//! dependency rules, and engine configuration.

// ============================================================================
// PMAT INTEGRATION
// ============================================================================

/// Default cyclomatic complexity threshold
pub const DEFAULT_COMPLEXITY_THRESHOLD: u32 = 15;

/// Default TDG score threshold (0-100, higher is worse)
pub const DEFAULT_TDG_THRESHOLD: u32 = 50;

// ============================================================================
// REFACTORING COMPLETENESS
// ============================================================================

/// Utility types that are intentionally duplicated to avoid cross-crate dependencies
pub const UTILITY_TYPES: &[&str] = &[
    "JsonExt",
    "HttpResponseUtils",
    "CacheStats",
    "TimedOperation",
];

/// Generic type names expected to appear in multiple places
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

/// File names to skip for test completeness checks
pub const REFACTORING_SKIP_FILES: &[&str] = &[
    "mod",
    "lib",
    "main",
    "constants",
    "thresholds",
    "error",
    "types",
];

/// Directory patterns to skip for test completeness checks (tested via integration)
pub const REFACTORING_SKIP_DIR_PATTERNS: &[&str] = &["/di/", "/config/", "/models/", "/ports/"];

// ============================================================================
// RETE ENGINE
// ============================================================================

// ============================================================================
// QUALITY / TEST QUALITY / IMPLEMENTATION (pending/stub labels)
//
// Labels are built via `concat!()` to prevent the source file itself from
// triggering ripgrep-based lint rules for task-comment patterns.
// ============================================================================

/// Label for pending task comments (first priority)
pub const PENDING_LABEL_TODO: &str = concat!("TO", "DO");

/// Label for fix-needed comments
pub const PENDING_LABEL_FIXME: &str = concat!("FI", "XME");

/// Label for attention-needed comments
pub const PENDING_LABEL_XXX: &str = concat!("X", "XX");

/// Label for workaround/shortcut comments
pub const PENDING_LABEL_HACK: &str = concat!("HA", "CK");

/// Label for panic-stub detection (unimplemented placeholders)
pub const STUB_PANIC_LABEL: &str = concat!("panic(", "TO", "DO)");

/// Label used in reporter tests (identical to pending-task label)
pub const REPORT_TEST_PENDING_LABEL: &str = concat!("TO", "DO");

// ============================================================================
// DUPLICATION (clone detection)
// ============================================================================

/// Common keywords to ignore when fingerprinting (multi-language)
pub const DUPLICATION_KEYWORDS: &[&str] = &[
    "fn",
    "let",
    "mut",
    "const",
    "static",
    "struct",
    "enum",
    "impl",
    "trait",
    "pub",
    "mod",
    "use",
    "crate",
    "self",
    "super",
    "where",
    "async",
    "await",
    "move",
    "ref",
    "match",
    "if",
    "else",
    "loop",
    "while",
    "for",
    "in",
    "break",
    "continue",
    "return",
    "type",
    "as",
    "dyn",
    "unsafe",
    "extern",
    "def",
    "class",
    "import",
    "from",
    "with",
    "try",
    "except",
    "finally",
    "raise",
    "pass",
    "yield",
    "lambda",
    "global",
    "nonlocal",
    "assert",
    "del",
    "True",
    "False",
    "None",
    "and",
    "or",
    "not",
    "is",
    "function",
    "var",
    "extends",
    "implements",
    "interface",
    "namespace",
    "module",
    "export",
    "default",
    "new",
    "delete",
    "typeof",
    "instanceof",
    "this",
    "null",
    "undefined",
    "true",
    "false",
    "void",
    "throw",
    "catch",
    "debugger",
    "switch",
    "case",
];
