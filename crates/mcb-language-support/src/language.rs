//! Language Identification and Registry
//!
//! Defines the `LanguageId` enum for supported programming languages
//! and `LanguageRegistry` for language lookup and metadata.

use std::collections::HashMap;

use rust_code_analysis::LANG;
use serde::{Deserialize, Serialize};

/// Supported programming languages
///
/// Maps to Mozilla rust-code-analysis LANG enum while providing
/// a stable, serializable representation. Only languages that RCA
/// actually supports are included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript (including JSX)
    JavaScript,
    /// TypeScript (including TSX)
    TypeScript,
    /// Java
    Java,
    /// C and C++ (RCA treats them identically)
    Cpp,
    /// Kotlin
    Kotlin,
}

impl LanguageId {
    /// Get all supported languages
    pub fn all() -> &'static [LanguageId] {
        &[
            LanguageId::Rust,
            LanguageId::Python,
            LanguageId::JavaScript,
            LanguageId::TypeScript,
            LanguageId::Java,
            LanguageId::Cpp,
            LanguageId::Kotlin,
        ]
    }

    /// Get the canonical name of the language
    pub fn name(&self) -> &'static str {
        match self {
            LanguageId::Rust => "rust",
            LanguageId::Python => "python",
            LanguageId::JavaScript => "javascript",
            LanguageId::TypeScript => "typescript",
            LanguageId::Java => "java",
            LanguageId::Cpp => "cpp",
            LanguageId::Kotlin => "kotlin",
        }
    }

    /// Get the display name of the language
    pub fn display_name(&self) -> &'static str {
        match self {
            LanguageId::Rust => "Rust",
            LanguageId::Python => "Python",
            LanguageId::JavaScript => "JavaScript",
            LanguageId::TypeScript => "TypeScript",
            LanguageId::Java => "Java",
            LanguageId::Cpp => "C/C++",
            LanguageId::Kotlin => "Kotlin",
        }
    }

    /// Get common file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            LanguageId::Rust => &["rs"],
            LanguageId::Python => &["py", "pyi", "pyw"],
            LanguageId::JavaScript => &["js", "mjs", "cjs", "jsx"],
            LanguageId::TypeScript => &["ts", "mts", "cts", "tsx"],
            LanguageId::Java => &["java"],
            LanguageId::Cpp => &["c", "h", "cpp", "cc", "cxx", "hpp", "hxx", "mm", "m"],
            LanguageId::Kotlin => &["kt", "kts"],
        }
    }

    /// Convert to rust-code-analysis LANG enum
    ///
    /// Some languages may map to different RCA variants (e.g., JavaScript to Mozjs).
    pub fn to_rca_lang(&self) -> LANG {
        match self {
            LanguageId::Rust => LANG::Rust,
            LanguageId::Python => LANG::Python,
            LanguageId::JavaScript => LANG::Mozjs,
            LanguageId::TypeScript => LANG::Typescript,
            LanguageId::Java => LANG::Java,
            LanguageId::Cpp => LANG::Cpp,
            LanguageId::Kotlin => LANG::Kotlin,
        }
    }

    /// Create from rust-code-analysis LANG enum
    pub fn from_rca_lang(lang: LANG) -> Option<LanguageId> {
        match lang {
            LANG::Rust => Some(LanguageId::Rust),
            LANG::Python => Some(LanguageId::Python),
            LANG::Mozjs | LANG::Javascript => Some(LanguageId::JavaScript),
            LANG::Typescript | LANG::Tsx => Some(LanguageId::TypeScript),
            LANG::Java => Some(LanguageId::Java),
            LANG::Cpp => Some(LanguageId::Cpp),
            LANG::Kotlin => Some(LanguageId::Kotlin),
            // Intentional: RCA supports more languages than we map to LanguageId
            other => {
                let _ = other;
                None
            }
        }
    }

    /// Try to create from a string name
    pub fn from_name(name: &str) -> Option<LanguageId> {
        match name.to_lowercase().as_str() {
            "rust" | "rs" => Some(LanguageId::Rust),
            "python" | "py" => Some(LanguageId::Python),
            "javascript" | "js" | "jsx" | "mozjs" => Some(LanguageId::JavaScript),
            "typescript" | "ts" | "tsx" => Some(LanguageId::TypeScript),
            "java" => Some(LanguageId::Java),
            "c" | "cpp" | "c++" | "cxx" => Some(LanguageId::Cpp),
            "kotlin" | "kt" => Some(LanguageId::Kotlin),
            // Intentional: return None for unrecognized language names
            other => {
                let _ = other;
                None
            }
        }
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Language metadata and capabilities
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    /// Language identifier
    pub id: LanguageId,
    /// Whether the language supports AST parsing
    pub supports_ast: bool,
    /// Whether the language supports metrics analysis
    pub supports_metrics: bool,
    /// Typical comment prefix (for single-line comments)
    pub comment_prefix: Option<&'static str>,
    /// Block comment delimiters (start, end)
    pub block_comment: Option<(&'static str, &'static str)>,
}

impl LanguageInfo {
    /// Create language info for a given language ID
    pub fn new(id: LanguageId) -> Self {
        let (comment_prefix, block_comment) = match id {
            LanguageId::Rust
            | LanguageId::JavaScript
            | LanguageId::TypeScript
            | LanguageId::Java
            | LanguageId::Cpp
            | LanguageId::Kotlin => (Some("//"), Some(("/*", "*/"))),
            LanguageId::Python => (Some("#"), None),
        };

        Self {
            id,
            supports_ast: true,     // All our languages support AST via RCA
            supports_metrics: true, // All our languages support metrics via RCA
            comment_prefix,
            block_comment,
        }
    }
}

/// Registry for language lookup and metadata
pub struct LanguageRegistry {
    /// Extension to language mapping
    extension_map: HashMap<&'static str, LanguageId>,
    /// Language info cache
    info_cache: HashMap<LanguageId, LanguageInfo>,
}

impl LanguageRegistry {
    /// Create a new language registry with all supported languages
    pub fn new() -> Self {
        let mut extension_map = HashMap::new();
        let mut info_cache = HashMap::new();

        for &lang_id in LanguageId::all() {
            for ext in lang_id.extensions() {
                extension_map.insert(*ext, lang_id);
            }
            info_cache.insert(lang_id, LanguageInfo::new(lang_id));
        }

        Self {
            extension_map,
            info_cache,
        }
    }

    /// Look up language by file extension
    pub fn by_extension(&self, ext: &str) -> Option<LanguageId> {
        let ext_lower = ext.to_lowercase();
        let ext_clean = ext_lower.trim_start_matches('.');
        self.extension_map.get(ext_clean).copied()
    }

    /// Get language info for a language ID
    ///
    /// Returns `None` if the language ID is not registered (should not happen
    /// for valid `LanguageId` values since all are populated in `new()`).
    pub fn info(&self, id: LanguageId) -> Option<&LanguageInfo> {
        self.info_cache.get(&id)
    }

    /// Get all registered extensions
    pub fn all_extensions(&self) -> Vec<&'static str> {
        self.extension_map.keys().copied().collect()
    }

    /// Check if an extension is supported
    pub fn is_supported_extension(&self, ext: &str) -> bool {
        self.by_extension(ext).is_some()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
