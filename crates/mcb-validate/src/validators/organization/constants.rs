//! Constants for organization validators.

// ============================================================================
// MAGIC NUMBERS (magic_numbers.rs)
// ============================================================================

/// Regex pattern for detecting 5+ digit magic numbers.
pub const MAGIC_NUMBER_REGEX: &str = r"\b(\d{5,})\b";

/// Allowed numeric literals (powers of 2, memory sizes, time values).
pub const ALLOWED_MAGIC_NUMBERS: &[&str] = &[
    // Powers of 2
    "16384",
    "32768",
    "65535",
    "65536",
    "131072",
    "262144",
    "524288",
    "1048576",
    "2097152",
    "4194304",
    // Common memory/count sizes
    "100000",
    "1000000",
    "10000000",
    "100000000",
    // Time values (seconds)
    "86400",
    "604800",
    "2592000",
    "31536000",
];

// ============================================================================
// DUPLICATE STRINGS (duplicate_strings.rs)
// ============================================================================

/// Regex for extracting string literals (15+ characters).
pub const DUPLICATE_STRING_REGEX: &str = r#""([^"\\]{15,})""#;

/// Minimum number of files a string must appear in to be flagged.
pub const DUPLICATE_STRING_MIN_FILES: usize = 4;

/// Patterns in string values that are OK to repeat across files.
pub const DUPLICATE_STRING_SKIP_PATTERNS: &[&str] = &[
    "{}",    // Format strings
    "test_", // Test names
    "Error", // Error message prefixes
    "error",
    "Failed",
    "Invalid",
    "Cannot",
    "Unable",
    "Missing",
    "://", // URLs
    ".rs", // File paths
    ".json",
    ".toml",
    "_id",  // ID fields
    "_key", // Key fields
    "pub ", // Code patterns
    "fn ",
    "let ",
    "CARGO_",       // env!() macros
    "serde_json",   // Code patterns
    ".to_string()", // Method chains
];

// ============================================================================
// DOMAIN PURITY (domain_purity.rs)
// ============================================================================

/// Allowed method names in domain impl blocks (constructors, accessors, conversions).
pub const DOMAIN_ALLOWED_METHODS: &[&str] = &[
    "new",
    "default",
    "definition",
    "tables",
    "fts_def",
    "indexes",
    "foreign_keys",
    "unique_constraints",
    "from",
    "into",
    "as_ref",
    "as_mut",
    "clone",
    "fmt",
    "eq",
    "cmp",
    "hash",
    "partial_cmp",
    "is_empty",
    "len",
    "iter",
    "into_iter",
    "total_changes",
    "from_ast",
    "from_fallback",
    "directory",
    "file",
    "sorted",
    "sort_children",
];

/// Allowed method name prefixes in domain impl blocks.
pub const DOMAIN_ALLOWED_PREFIXES: &[&str] = &[
    "from_", "into_", "as_", "to_", "get_", "is_", "has_", "with_",
];

/// Path fragment identifying the domain crate.
pub const DOMAIN_CRATE_PATH: &str = "domain";

/// Path fragment identifying the ports directory (skip in domain purity check).
pub const PORTS_DIR_PATH: &str = "/ports/";

// ============================================================================
// LAYER VIOLATIONS (layer_violations.rs)
// ============================================================================

/// Regex for detecting direct service instantiation via `Arc::new(Service::new`.
pub const ARC_NEW_SERVICE_REGEX: &str =
    r"Arc::new\s*\(\s*([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository))::new";

/// Regex for detecting server-layer imports.
pub const SERVER_IMPORT_REGEX: &str = r"use\s+(?:crate::|super::)*server::";

/// Path fragment identifying the server layer.
pub const SERVER_LAYER_PATH: &str = "/server/";

/// Path fragment identifying the application layer.
pub const APPLICATION_LAYER_PATH: &str = "/application/";

/// Path fragment identifying the infrastructure layer.
pub const INFRASTRUCTURE_LAYER_PATH: &str = "/infrastructure/";

/// File names that are allowed to bypass the direct service creation rule.
pub const SERVICE_CREATION_BYPASS_FILES: &[&str] = &["builder", "factory", "bootstrap"];
