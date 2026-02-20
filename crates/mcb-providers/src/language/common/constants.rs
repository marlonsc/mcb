//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Constants for code chunking operations
//!
//! Language-specific chunk sizes and node extraction rule defaults.

use mcb_domain::constants::lang::{
    LANG_C, LANG_CPP, LANG_CSHARP, LANG_GO, LANG_JAVA, LANG_JAVASCRIPT, LANG_KOTLIN, LANG_PHP,
    LANG_PYTHON, LANG_RUBY, LANG_RUST, LANG_SWIFT, LANG_TYPESCRIPT,
};

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
// Tree-Sitter Node Types -- re-exported from mcb-domain (Single Source of Truth)
// ============================================================================
pub use mcb_domain::constants::ast::*;

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
