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

/// Known migration pairs where duplicates are expected temporarily
pub const KNOWN_MIGRATION_PAIRS: &[(&str, &str)] = &[
    ("mcb-providers", "mcb-infrastructure"),
    ("mcb-domain", "mcb-infrastructure"),
];

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

// ============================================================================
// DEPENDENCY GRAPH (CLEAN ARCHITECTURE)
// ============================================================================

/// Allowed dependencies per crate for Clean Architecture layer boundaries
pub const ALLOWED_DEPS: &[(&str, &[&str])] = &[
    ("mcb-domain", &[]),
    ("mcb-application", &["mcb-domain"]),
    ("mcb-providers", &["mcb-domain", "mcb-application"]),
    (
        "mcb-infrastructure",
        &[
            "mcb-domain",
            "mcb-application",
            "mcb-providers",
            "mcb-validate",
        ],
    ),
    (
        "mcb-server",
        &[
            "mcb-domain",
            "mcb-application",
            "mcb-infrastructure",
            "mcb-providers",
        ],
    ),
    (
        "mcb",
        &[
            "mcb-domain",
            "mcb-application",
            "mcb-infrastructure",
            "mcb-server",
            "mcb-providers",
            "mcb-validate",
        ],
    ),
    ("mcb-validate", &["mcb-language-support", "mcb-ast-utils"]),
    ("mcb-language-support", &[]),
    ("mcb-ast-utils", &["mcb-language-support"]),
];

// ============================================================================
// RETE ENGINE
// ============================================================================

/// Prefix for internal workspace dependencies (mcb-*)
pub const INTERNAL_DEP_PREFIX: &str = "mcb-";

// ============================================================================
// REFACTORING (missing test files)
// ============================================================================

/// Source file stems that don't require dedicated unit test files
pub const REFACTORING_SKIP_FILES: &[&str] = &[
    "mod",
    "lib",
    "main",
    "prelude",
    "constants",
    "types",
    "error",
    "errors",
    "helpers",
    "utils",
    "common",
    "config",
    "builder",
    "factory",
    "indexing",
    "search_repository",
    "metrics",
    "components",
    "operations",
    "rate_limit_middleware",
    "security",
    "mcp_server",
    "init",
];

/// Directory names that are tested via integration tests
pub const REFACTORING_SKIP_DIR_PATTERNS: &[&str] = &[
    "providers",
    "adapters",
    "language",
    "embedding",
    "vector_store",
    "cache",
    "hybrid_search",
    "events",
    "chunking",
    "http",
    "di",
    "admin",
    "handlers",
    "config",
    "tools",
    "utils",
    "ports",
];

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
