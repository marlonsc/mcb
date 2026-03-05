//! Detection patterns: quality, SOLID, KISS, refactoring, implementation,
//! documentation, async, performance, and organization.

use super::patterns::CONTEXT_PREVIEW_LENGTH;

// ============================================================================
// Quality Detection (unwrap/panic)
// ============================================================================

/// Safety justification comment markers.
pub const SAFETY_COMMENT_MARKERS: &[&str] = &["// SAFETY:", "// safety:"];

/// Ignore hint keywords for unwrap/expect suppression.
pub const IGNORE_HINT_KEYWORDS: &[&str] = &["lock_poisoning_recovery"];

/// Number of lines before/after a detection to search for ignore hints.
pub const COMMENT_SEARCH_RADIUS: usize = 3;

/// Strings that indicate legitimate lock-poisoning `expect()` usage.
pub const LOCK_POISONING_STRINGS: &[&str] = &[
    "lock poisoned",
    "Lock poisoned",
    "poisoned",
    "Mutex poisoned",
];

/// Regex pattern for detecting `panic!()` macro usage.
pub const PANIC_REGEX: &str = r"panic!\s*\(";

// ============================================================================
// SOLID Detection
// ============================================================================

/// Max unrelated structs in a single file before SRP warning.
pub const MAX_UNRELATED_STRUCTS_PER_FILE: usize = 5;

/// Min string-based match arms before OCP dispatch warning.
pub const MIN_STRING_MATCH_ARMS_FOR_DISPATCH: usize = 3;

/// Min names needed for relationship check.
pub const MIN_NAMES_FOR_RELATION_CHECK: usize = 2;

/// Min shared prefix/suffix length for relationship detection.
pub const MIN_AFFIX_LENGTH: usize = 3;

/// Max shared prefix/suffix length for relationship detection.
pub const MAX_AFFIX_LENGTH: usize = 10;

/// Min word length for semantic comparison in CamelCase splitting.
pub const MIN_WORD_LENGTH_FOR_COMPARISON: usize = 4;

// ============================================================================
// KISS Detection
// ============================================================================

/// Type name suffixes that identify DI container structs (allowed more fields).
pub const DI_CONTAINER_SUFFIXES: &[&str] = &["Context", "Container", "Components", "State"];

/// Type name substrings that identify config-like structs (allowed more fields).
pub const DI_CONTAINER_CONTAINS: &[&str] = &["Config", "Settings"];

/// Minimum line distance between reported nesting violations to avoid noise.
pub const NESTING_PROXIMITY_THRESHOLD: usize = 5;

// ============================================================================
// Refactoring Detection
// ============================================================================

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

// ============================================================================
// Implementation Detection
// ============================================================================

/// Hardcoded return pattern IDs and descriptions: (`pattern_id`, description).
pub const HARDCODED_RETURN_PATTERNS: &[(&str, &str)] = &[
    ("IMPL001.return_true", "true"),
    ("IMPL001.return_false", "false"),
    ("IMPL001.return_zero", "0"),
    ("IMPL001.return_one", "1"),
    ("IMPL001.return_empty_string", "empty string"),
    ("IMPL001.return_hardcoded_string", "hardcoded string"),
];

/// File names to skip in hardcoded return detection.
pub const STUB_SKIP_FILE_KEYWORDS: &[&str] = &["null", "fake", "constants.rs"];

// ============================================================================
// Documentation Detection
// ============================================================================

/// Regex for detecting doc comments (`///`).
pub const DOC_COMMENT_REGEX: &str = r"^\s*///";

/// Regex for capturing doc comment content after `///`.
pub const DOC_COMMENT_CAPTURE_REGEX: &str = r"^\s*///(.*)";

/// Regex for detecting attributes (`#[...]`).
pub const ATTR_REGEX: &str = r"^\s*#\[";

/// Regex for detecting module-level doc comments (`//!`).
pub const MODULE_DOC_REGEX: &str = "^//!";

/// Regex for detecting `pub struct` declarations.
pub const PUB_STRUCT_REGEX: &str = r"pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub enum` declarations.
pub const PUB_ENUM_REGEX: &str = r"pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub trait` declarations.
pub const PUB_TRAIT_REGEX: &str = r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub fn` / `pub async fn` declarations.
pub const PUB_FN_REGEX: &str = r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)";

/// Regex for detecting example sections in documentation.
pub const EXAMPLE_SECTION_REGEX: &str = r"#\s*Example";

/// File names that require module-level documentation.
pub const MODULE_FILE_NAMES: &[&str] = &["lib.rs", "mod.rs"];

/// Paths identifying DI module traits (skip example checking).
pub const DI_MODULES_PATH: &str = "/di/modules/";

/// Label for struct items in violation messages.
pub const ITEM_KIND_STRUCT: &str = "struct";

/// Label for enum items in violation messages.
pub const ITEM_KIND_ENUM: &str = "enum";

/// Label for trait items in violation messages.
pub const ITEM_KIND_TRAIT: &str = "trait";

/// Label for function items in violation messages.
pub const ITEM_KIND_FUNCTION: &str = "function";

// ============================================================================
// Async Pattern Detection
// ============================================================================

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

// ============================================================================
// Performance Detection
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

// ============================================================================
// Organization Detection
// ============================================================================

/// Regex pattern for detecting 5+ digit magic numbers.
pub const MAGIC_NUMBER_REGEX: &str = r"\b(\d{5,})\b";

/// Allowed numeric literals (powers of 2, memory sizes, time values).
pub const ALLOWED_MAGIC_NUMBERS: &[&str] = &[
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
    "100000",
    "1000000",
    "10000000",
    "100000000",
    "86400",
    "604800",
    "2592000",
    "31536000",
];

/// Regex for extracting string literals (15+ characters).
pub const DUPLICATE_STRING_REGEX: &str = r#""([^"\\]{15,})""#;

/// Minimum number of files a string must appear in to be flagged.
pub const DUPLICATE_STRING_MIN_FILES: usize = 4;

/// Patterns in string values that are OK to repeat across files.
pub const DUPLICATE_STRING_SKIP_PATTERNS: &[&str] = &[
    "{}",
    "test_",
    "Error",
    "error",
    "Failed",
    "Invalid",
    "Cannot",
    "Unable",
    "Missing",
    "://",
    ".rs",
    ".json",
    ".toml",
    "_id",
    "_key",
    "pub ",
    "fn ",
    "let ",
    "CARGO_",
    "serde_json",
    ".to_string()",
];

/// Allowed method names in domain impl blocks.
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
    "success",
    "error",
    "render",
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
