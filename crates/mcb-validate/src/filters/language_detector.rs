//! Language Detection for Rule Filtering
//!
//! Detects programming language from file extensions and content patterns.
//! Uses `rust-code-analysis` directly for language detection.
//!
//! Previously delegated to the `mcb-language-support` crate; all logic is now
//! inlined here to remove that dependency.

use std::path::Path;
use std::str::FromStr;

use rust_code_analysis::{LANG, guess_language};

/// Supported programming languages.
///
/// Maps to `rust_code_analysis::LANG` while providing a stable,
/// serializable representation. Only languages that RCA actually
/// supports are included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    /// C / C++ (RCA treats them identically)
    Cpp,
    /// Kotlin
    Kotlin,
}

impl LanguageId {
    /// Get the canonical name of the language.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Java => "java",
            Self::Cpp => "cpp",
            Self::Kotlin => "kotlin",
        }
    }

    /// Convert to `rust_code_analysis::LANG`.
    pub fn to_rca_lang(&self) -> LANG {
        match self {
            Self::Rust => LANG::Rust,
            Self::Python => LANG::Python,
            Self::JavaScript => LANG::Mozjs,
            Self::TypeScript => LANG::Typescript,
            Self::Java => LANG::Java,
            Self::Cpp => LANG::Cpp,
            Self::Kotlin => LANG::Kotlin,
        }
    }

    /// Create from `rust_code_analysis::LANG`.
    pub fn from_rca_lang(lang: LANG) -> Option<Self> {
        match lang {
            LANG::Rust => Some(Self::Rust),
            LANG::Python => Some(Self::Python),
            LANG::Mozjs | LANG::Javascript => Some(Self::JavaScript),
            LANG::Typescript | LANG::Tsx => Some(Self::TypeScript),
            LANG::Java => Some(Self::Java),
            LANG::Cpp => Some(Self::Cpp),
            LANG::Kotlin => Some(Self::Kotlin),
            // RCA supports more languages than we map to LanguageId
            _other => None,
        }
    }

    /// Common file extensions for this language.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["rs"],
            Self::Python => &["py", "pyi", "pyw"],
            Self::JavaScript => &["js", "mjs", "cjs", "jsx"],
            Self::TypeScript => &["ts", "mts", "cts", "tsx"],
            Self::Java => &["java"],
            Self::Cpp => &["c", "h", "cpp", "cc", "cxx", "hpp", "hxx", "mm", "m"],
            Self::Kotlin => &["kt", "kts"],
        }
    }

    /// Try to create from a string name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "javascript" | "js" | "jsx" | "mozjs" => Some(Self::JavaScript),
            "typescript" | "ts" | "tsx" => Some(Self::TypeScript),
            "java" => Some(Self::Java),
            "cpp" | "c++" | "c" | "cxx" => Some(Self::Cpp),
            "kotlin" | "kt" => Some(Self::Kotlin),
            _ => None,
        }
    }
}

impl FromStr for LanguageId {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_name(s).ok_or_else(|| format!("Unknown language: {s}"))
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Detects programming language from file paths using `rust_code_analysis::guess_language`.
pub struct LanguageDetector {
    _private: (),
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetector {
    /// Create a new language detector.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Detect language from file path, returning a [`LanguageId`].
    pub fn detect(&self, path: &Path, content: Option<&str>) -> Option<LanguageId> {
        let source = content.map_or_else(
            || std::fs::read(path).unwrap_or_default(),
            |c| c.as_bytes().to_vec(),
        );
        let (rca_lang, _) = guess_language(&source, path);
        rca_lang.and_then(LanguageId::from_rca_lang)
    }

    /// Detect language and return as string name.
    pub fn detect_name(&self, path: &Path, content: Option<&str>) -> Option<String> {
        self.detect(path, content).map(|id| id.name().to_string())
    }

    /// Detect language and return `LANG` enum for direct RCA usage.
    pub fn detect_rca_lang(&self, path: &Path, content: Option<&str>) -> Option<LANG> {
        self.detect(path, content).map(|id| id.to_rca_lang())
    }

    /// All supported language names.
    pub fn supported_language_names(&self) -> Vec<String> {
        vec![
            "rust".to_string(),
            "python".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
            "java".to_string(),
            "cpp".to_string(),
            "kotlin".to_string(),
        ]
    }

    /// Check if a file matches any of the specified language names.
    pub fn matches_languages(
        &self,
        path: &Path,
        content: Option<&str>,
        allowed_languages: &[String],
    ) -> bool {
        self.detect_name(path, content)
            .is_some_and(|language| allowed_languages.contains(&language))
    }
}
