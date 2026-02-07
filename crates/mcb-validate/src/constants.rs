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

/// Known migration pairs where duplicates are expected temporarily.
/// DEPRECATED: Configure migration pairs in .mcb-validate.toml instead:
/// ```toml
/// [rules.refactoring]
/// known_migration_pairs = [["old-crate", "new-crate"]]
/// ```
#[deprecated(note = "Use FileConfig::rules.refactoring.known_migration_pairs instead")]
pub const KNOWN_MIGRATION_PAIRS: &[(&str, &str)] = &[];

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
// RETE ENGINE
// ============================================================================

/// Prefix for internal workspace dependencies.
/// DEPRECATED: Configure internal_dep_prefix in .mcb-validate.toml instead:
/// ```toml
/// [general]
/// internal_dep_prefix = "myapp-"
/// ```
#[deprecated(note = "Use FileConfig::general.internal_dep_prefix instead")]
pub const INTERNAL_DEP_PREFIX: &str = "";

// ============================================================================
// DUPLICATION (clone detection)
// ============================================================================

// ============================================================================
// DUPLICATION (clone detection)
// ============================================================================

// ============================================================================
// QUALITY / TEST QUALITY / IMPLEMENTATION (pending/stub labels)
// ============================================================================

/// Label for TODO comments (quality checks)
pub const PENDING_LABEL_TODO: &str = "TODO";

/// Label for FIXME comments
pub const PENDING_LABEL_FIXME: &str = "FIXME";

/// Label for XXX comments
pub const PENDING_LABEL_XXX: &str = "XXX";

/// Label for HACK comments
pub const PENDING_LABEL_HACK: &str = "HACK";

/// Label for panic(TODO) stub detection
pub const STUB_PANIC_LABEL: &str = "panic(TODO)";

/// Label used in reporter tests (same as TODO, avoids literal in test source)
pub const REPORT_TEST_PENDING_LABEL: &str = "TODO";

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
