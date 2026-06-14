//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Language identifier and chunking constants -- Single Source of Truth
//!
//! All language string identifiers used across the codebase.

/// JavaScript language identifier
pub const LANG_JAVASCRIPT: &str = "javascript";
/// TypeScript language identifier
pub const LANG_TYPESCRIPT: &str = "typescript";
/// Python language identifier
pub const LANG_PYTHON: &str = "python";
/// Rust language identifier
pub const LANG_RUST: &str = "rust";
/// Go language identifier
pub const LANG_GO: &str = "go";
/// Java language identifier
pub const LANG_JAVA: &str = "java";
/// C language identifier
pub const LANG_C: &str = "c";
/// C++ language identifier
pub const LANG_CPP: &str = "cpp";
/// C# language identifier
pub const LANG_CSHARP: &str = "csharp";
/// Ruby language identifier
pub const LANG_RUBY: &str = "ruby";
/// PHP language identifier
pub const LANG_PHP: &str = "php";
/// Swift language identifier
pub const LANG_SWIFT: &str = "swift";
/// Kotlin language identifier
pub const LANG_KOTLIN: &str = "kotlin";
/// Unknown/unsupported language identifier
pub const LANG_UNKNOWN: &str = "unknown";

/// Maximum chunks per file for language processing.
pub const LANGUAGE_MAX_CHUNKS_PER_FILE: usize = 75;

/// Priority threshold for chunk filtering.
pub const LANGUAGE_PRIORITY_THRESHOLD: usize = 50;

// ============================================================================
// Default Chunk Sizes
// ============================================================================

/// Default lines per code chunk (fallback when no language-specific size).
pub const DEFAULT_CHUNK_SIZE: usize = 20;

/// Rust language chunk size.
pub const CHUNK_SIZE_RUST: usize = 20;

/// Python language chunk size.
pub const CHUNK_SIZE_PYTHON: usize = 15;

/// JavaScript/TypeScript language chunk size.
pub const CHUNK_SIZE_JAVASCRIPT: usize = 15;

/// Go language chunk size.
pub const CHUNK_SIZE_GO: usize = 15;

/// Java language chunk size.
pub const CHUNK_SIZE_JAVA: usize = 15;

/// C language chunk size.
pub const CHUNK_SIZE_C: usize = 15;

/// C++ language chunk size.
pub const CHUNK_SIZE_CPP: usize = 15;

/// C# language chunk size.
pub const CHUNK_SIZE_CSHARP: usize = 15;

/// Ruby language chunk size.
pub const CHUNK_SIZE_RUBY: usize = 15;

/// PHP language chunk size.
pub const CHUNK_SIZE_PHP: usize = 15;

/// Swift language chunk size.
pub const CHUNK_SIZE_SWIFT: usize = 15;

/// Kotlin language chunk size.
pub const CHUNK_SIZE_KOTLIN: usize = 15;

/// Generic/fallback language chunk size (for unsupported languages).
pub const CHUNK_SIZE_GENERIC: usize = 15;

// ============================================================================
// Node Extraction Rules Configuration
// ============================================================================

/// Node extraction rule default minimum content length.
pub const NODE_EXTRACTION_MIN_LENGTH: usize = 20;

/// Node extraction rule default minimum lines.
pub const NODE_EXTRACTION_MIN_LINES: usize = 1;

/// Node extraction rule default maximum depth.
pub const NODE_EXTRACTION_MAX_DEPTH: usize = 3;

/// Node extraction rule default priority.
pub const NODE_EXTRACTION_DEFAULT_PRIORITY: i32 = 5;

// ============================================================================
// Extension and Chunk Size Mapping Tables
// ============================================================================

/// Extension to language identifier mapping (used by detection).
pub const EXTENSION_LANG_MAP: &[(&[&str], &str)] = &[
    (&["rs"], LANG_RUST),
    (&["py", "pyw", "pyi"], LANG_PYTHON),
    (&["js", "mjs", "cjs", "jsx"], LANG_JAVASCRIPT),
    (&["ts", "tsx", "mts", "cts"], LANG_TYPESCRIPT),
    (&["go"], LANG_GO),
    (&["java"], LANG_JAVA),
    (&["c", "h"], LANG_C),
    (&["cpp", "cc", "cxx", "hpp", "hxx", "hh"], LANG_CPP),
    (&["cs"], LANG_CSHARP),
    (&["rb", "rake", "gemspec"], LANG_RUBY),
    (&["php", "phtml"], LANG_PHP),
    (&["swift"], LANG_SWIFT),
    (&["kt", "kts"], LANG_KOTLIN),
];

/// Language to chunk size mapping (used by detection).
pub const LANG_CHUNK_SIZE_MAP: &[(&[&str], usize)] = &[
    (&[LANG_RUST], CHUNK_SIZE_RUST),
    (&[LANG_PYTHON], CHUNK_SIZE_PYTHON),
    (&[LANG_JAVASCRIPT, LANG_TYPESCRIPT], CHUNK_SIZE_JAVASCRIPT),
    (&[LANG_GO], CHUNK_SIZE_GO),
    (&[LANG_JAVA], CHUNK_SIZE_JAVA),
    (&[LANG_C], CHUNK_SIZE_C),
    (&[LANG_CPP], CHUNK_SIZE_CPP),
    (&[LANG_CSHARP], CHUNK_SIZE_CSHARP),
    (&[LANG_RUBY], CHUNK_SIZE_RUBY),
    (&[LANG_PHP], CHUNK_SIZE_PHP),
    (&[LANG_SWIFT], CHUNK_SIZE_SWIFT),
    (&[LANG_KOTLIN], CHUNK_SIZE_KOTLIN),
];
