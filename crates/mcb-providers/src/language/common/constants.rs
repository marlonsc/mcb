//! Constants for code chunking operations
//!
//! Language-specific chunk sizes and node extraction rule defaults.

// ============================================================================
// Default Chunk Size
// ============================================================================

/// Default lines per code chunk (fallback when no language-specific size)
pub const DEFAULT_CHUNK_SIZE: usize = 20;

// ============================================================================
// Language-Specific Chunk Sizes
// ============================================================================

/// Rust language chunk size
pub const CHUNK_SIZE_RUST: usize = 20;

/// Python language chunk size
pub const CHUNK_SIZE_PYTHON: usize = 15;

/// JavaScript/TypeScript language chunk size
pub const CHUNK_SIZE_JAVASCRIPT: usize = 15;

/// Go language chunk size
pub const CHUNK_SIZE_GO: usize = 15;

/// Java language chunk size
pub const CHUNK_SIZE_JAVA: usize = 15;

/// C language chunk size
pub const CHUNK_SIZE_C: usize = 15;

/// C++ language chunk size
pub const CHUNK_SIZE_CPP: usize = 15;

/// C# language chunk size
pub const CHUNK_SIZE_CSHARP: usize = 15;

/// Ruby language chunk size
pub const CHUNK_SIZE_RUBY: usize = 15;

/// PHP language chunk size
pub const CHUNK_SIZE_PHP: usize = 15;

/// Swift language chunk size
pub const CHUNK_SIZE_SWIFT: usize = 15;

/// Kotlin language chunk size
pub const CHUNK_SIZE_KOTLIN: usize = 15;

/// Generic/fallback language chunk size (for unsupported languages)
pub const CHUNK_SIZE_GENERIC: usize = 15;

// ============================================================================
// Node Extraction Rules Configuration
// ============================================================================

/// Node extraction rule default minimum content length
pub const NODE_EXTRACTION_MIN_LENGTH: usize = 20;

/// Node extraction rule default minimum lines
pub const NODE_EXTRACTION_MIN_LINES: usize = 1;

/// Node extraction rule default maximum depth
pub const NODE_EXTRACTION_MAX_DEPTH: usize = 3;

/// Node extraction rule default priority
pub const NODE_EXTRACTION_DEFAULT_PRIORITY: i32 = 5;

// ============================================================================
// Language Identifiers (String constants for extensibility)
// ============================================================================

/// Rust language identifier
pub const LANG_RUST: &str = "rust";

/// Python language identifier
pub const LANG_PYTHON: &str = "python";

/// JavaScript language identifier
pub const LANG_JAVASCRIPT: &str = "javascript";

/// TypeScript language identifier
pub const LANG_TYPESCRIPT: &str = "typescript";

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

// ============================================================================
// Tree-Sitter Node Types
// ============================================================================

/// Function declaration node type (for C-like languages)
pub const TS_NODE_FUNCTION_DECLARATION: &str = "function_declaration";

/// Function definition node type (for Python, etc.)
pub const TS_NODE_FUNCTION_DEFINITION: &str = "function_definition";

/// Method declaration node type (for OOP languages)
pub const TS_NODE_METHOD_DECLARATION: &str = "method_declaration";

/// Class declaration node type
pub const TS_NODE_CLASS_DECLARATION: &str = "class_declaration";

/// Interface declaration node type
pub const AST_NODE_INTERFACE_DECLARATION: &str = "interface_declaration";

/// Struct specifier node type (C/C++)
pub const AST_NODE_STRUCT_SPECIFIER: &str = "struct_specifier";

// ============================================================================
// Extension and chunk size mapping tables
// ============================================================================

/// Extension to language identifier mapping (used by detection)
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

/// Language to chunk size mapping (used by detection)
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
