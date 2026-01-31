//! Language Identification and Registry
//!
//! Defines the `LanguageId` enum for supported programming languages
//! and `LanguageRegistry` for language lookup and metadata.

use rust_code_analysis::LANG;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            _ => None,
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
            _ => None,
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
    pub fn info(&self, id: LanguageId) -> &LanguageInfo {
        // Safe because we populate info_cache for all languages in new()
        self.info_cache.get(&id).expect("Language info missing")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_id_all() {
        let all = LanguageId::all();
        assert_eq!(all.len(), 7);
        assert!(all.contains(&LanguageId::Rust));
        assert!(all.contains(&LanguageId::Kotlin));
    }

    #[test]
    fn test_language_id_name() {
        assert_eq!(LanguageId::Rust.name(), "rust");
        assert_eq!(LanguageId::Cpp.name(), "cpp");
        assert_eq!(LanguageId::JavaScript.name(), "javascript");
    }

    #[test]
    fn test_language_id_display_name() {
        assert_eq!(LanguageId::Cpp.display_name(), "C/C++");
        assert_eq!(LanguageId::JavaScript.display_name(), "JavaScript");
        assert_eq!(LanguageId::TypeScript.display_name(), "TypeScript");
    }

    #[test]
    fn test_language_id_extensions() {
        assert!(LanguageId::Rust.extensions().contains(&"rs"));
        assert!(LanguageId::Python.extensions().contains(&"py"));
        assert!(LanguageId::JavaScript.extensions().contains(&"jsx"));
        assert!(LanguageId::Cpp.extensions().contains(&"c"));
        assert!(LanguageId::Cpp.extensions().contains(&"cpp"));
    }

    #[test]
    fn test_language_id_from_name() {
        assert_eq!(LanguageId::from_name("rust"), Some(LanguageId::Rust));
        assert_eq!(LanguageId::from_name("PYTHON"), Some(LanguageId::Python));
        assert_eq!(LanguageId::from_name("c++"), Some(LanguageId::Cpp));
        assert_eq!(LanguageId::from_name("c"), Some(LanguageId::Cpp));
        assert_eq!(LanguageId::from_name("unknown"), None);
    }

    #[test]
    fn test_language_id_rca_conversion() {
        assert_eq!(LanguageId::Rust.to_rca_lang(), LANG::Rust);
        assert_eq!(LanguageId::JavaScript.to_rca_lang(), LANG::Mozjs);

        assert_eq!(
            LanguageId::from_rca_lang(LANG::Rust),
            Some(LanguageId::Rust)
        );
        assert_eq!(
            LanguageId::from_rca_lang(LANG::Mozjs),
            Some(LanguageId::JavaScript)
        );
    }

    #[test]
    fn test_language_registry_by_extension() {
        let registry = LanguageRegistry::new();

        assert_eq!(registry.by_extension("rs"), Some(LanguageId::Rust));
        assert_eq!(registry.by_extension(".py"), Some(LanguageId::Python));
        assert_eq!(registry.by_extension("JS"), Some(LanguageId::JavaScript));
        assert_eq!(registry.by_extension("unknown"), None);
    }

    #[test]
    fn test_language_registry_info() {
        let registry = LanguageRegistry::new();

        let rust_info = registry.info(LanguageId::Rust);
        assert!(rust_info.supports_ast);
        assert_eq!(rust_info.comment_prefix, Some("//"));
        assert_eq!(rust_info.block_comment, Some(("/*", "*/")));

        let python_info = registry.info(LanguageId::Python);
        assert_eq!(python_info.comment_prefix, Some("#"));
        assert_eq!(python_info.block_comment, None);
    }

    #[test]
    fn test_language_registry_all_extensions() {
        let registry = LanguageRegistry::new();
        let extensions = registry.all_extensions();

        assert!(extensions.contains(&"rs"));
        assert!(extensions.contains(&"py"));
        assert!(extensions.contains(&"kt"));
    }
}
