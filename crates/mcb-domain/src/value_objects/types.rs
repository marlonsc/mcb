//! Domain Type Definitions
//!
//! Type aliases and basic type definitions for dynamic domain concepts.
//! These allow the domain to be extended without changing core types.

use std::path::Path;

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Programming language identifier
///
/// A string-based identifier for programming languages that allows dynamic
/// extension without modifying the domain layer. Language support is determined
/// by the application and infrastructure layers.
pub type Language = String;

/// Supported programming languages with type-safe enumeration
///
/// This enum provides compile-time safety for language identification.
/// It maps 1:1 with RCA (rust-code-analysis) LANG enum for metrics support.
///
/// # Example
///
/// ```
/// use mcb_domain::value_objects::SupportedLanguage;
///
/// let lang = SupportedLanguage::from_extension("rs");
/// assert_eq!(lang, Some(SupportedLanguage::Rust));
///
/// let display = SupportedLanguage::Rust.as_str();
/// assert_eq!(display, "rust");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum SupportedLanguage {
    /// Rust programming language
    #[display("rust")]
    Rust,
    /// Python programming language
    #[display("python")]
    Python,
    /// JavaScript (including JSX)
    #[display("javascript")]
    JavaScript,
    /// TypeScript (including TSX)
    #[display("typescript")]
    TypeScript,
    /// Go programming language
    #[display("go")]
    Go,
    /// Java programming language
    #[display("java")]
    Java,
    /// C programming language
    #[display("c")]
    C,
    /// C++ programming language
    #[display("cpp")]
    Cpp,
    /// C# programming language
    #[display("csharp")]
    CSharp,
    /// Ruby programming language
    #[display("ruby")]
    Ruby,
    /// PHP programming language
    #[display("php")]
    Php,
    /// Swift programming language
    #[display("swift")]
    Swift,
    /// Kotlin programming language
    #[display("kotlin")]
    Kotlin,
}

impl SupportedLanguage {
    /// Get language from file extension
    ///
    /// # Arguments
    /// * `ext` - File extension without the dot (e.g., "rs", "py")
    ///
    /// # Returns
    /// The corresponding language, or None if not supported
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" | "pyi" | "pyw" => Some(Self::Python),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Some(Self::Cpp),
            "cs" => Some(Self::CSharp),
            "rb" | "rake" | "gemspec" => Some(Self::Ruby),
            "php" | "phtml" => Some(Self::Php),
            "swift" => Some(Self::Swift),
            "kt" | "kts" => Some(Self::Kotlin),
            // Intentional: return None for unsupported extensions (not an error)
            other => {
                let _ = other; // Acknowledge all extensions are considered
                None
            }
        }
    }

    /// Get language from file path
    ///
    /// # Arguments
    /// * `path` - Path to the file
    ///
    /// # Returns
    /// The corresponding language based on file extension, or None
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Get the string representation of this language
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Go => "go",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Swift => "swift",
            Self::Kotlin => "kotlin",
        }
    }

    /// Get file extensions for this language
    pub fn get_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["rs"],
            Self::Python => &["py", "pyi", "pyw"],
            Self::JavaScript => &["js", "jsx", "mjs", "cjs"],
            Self::TypeScript => &["ts", "tsx", "mts", "cts"],
            Self::Go => &["go"],
            Self::Java => &["java"],
            Self::C => &["c", "h"],
            Self::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx", "hh"],
            Self::CSharp => &["cs"],
            Self::Ruby => &["rb", "rake", "gemspec"],
            Self::Php => &["php", "phtml"],
            Self::Swift => &["swift"],
            Self::Kotlin => &["kt", "kts"],
        }
    }

    /// Get all supported languages
    pub fn get_all() -> &'static [Self] {
        &[
            Self::Rust,
            Self::Python,
            Self::JavaScript,
            Self::TypeScript,
            Self::Go,
            Self::Java,
            Self::C,
            Self::Cpp,
            Self::CSharp,
            Self::Ruby,
            Self::Php,
            Self::Swift,
            Self::Kotlin,
        ]
    }

    /// Check if this language supports metrics via RCA
    pub fn is_metrics_supported(&self) -> bool {
        // RCA supports all these languages
        true
    }
}

impl_from_str!(SupportedLanguage, "Unsupported language: {}", {
    "rust" => SupportedLanguage::Rust,
    "rs" => SupportedLanguage::Rust,
    "python" => SupportedLanguage::Python,
    "py" => SupportedLanguage::Python,
    "javascript" => SupportedLanguage::JavaScript,
    "js" => SupportedLanguage::JavaScript,
    "typescript" => SupportedLanguage::TypeScript,
    "ts" => SupportedLanguage::TypeScript,
    "go" => SupportedLanguage::Go,
    "golang" => SupportedLanguage::Go,
    "java" => SupportedLanguage::Java,
    "c" => SupportedLanguage::C,
    "cpp" => SupportedLanguage::Cpp,
    "c++" => SupportedLanguage::Cpp,
    "cxx" => SupportedLanguage::Cpp,
    "csharp" => SupportedLanguage::CSharp,
    "c#" => SupportedLanguage::CSharp,
    "cs" => SupportedLanguage::CSharp,
    "ruby" => SupportedLanguage::Ruby,
    "rb" => SupportedLanguage::Ruby,
    "php" => SupportedLanguage::Php,
    "swift" => SupportedLanguage::Swift,
    "kotlin" => SupportedLanguage::Kotlin,
    "kt" => SupportedLanguage::Kotlin,
});

/// System operation type identifier
///
/// A string-based identifier for operation types used in metrics and rate limiting.
/// Allows dynamic extension of operation types without domain changes.
pub type OperationType = String;

/// Embedding provider identifier
///
/// A string-based identifier for embedding providers that allows dynamic
/// extension without modifying the domain layer. Provider capabilities
/// are determined by the application and infrastructure layers.
pub type EmbeddingProviderKind = String;

/// Vector store provider identifier
///
/// A string-based identifier for vector store providers that allows dynamic
/// extension without modifying the domain layer. Provider capabilities
/// are determined by the application and infrastructure layers.
pub type VectorStoreProviderKind = String;

/// Cache provider identifier
///
/// A string-based identifier for cache providers that allows dynamic
/// extension without modifying the domain layer. Provider capabilities
/// are determined by the application and infrastructure layers.
pub type CacheProviderKind = String;
