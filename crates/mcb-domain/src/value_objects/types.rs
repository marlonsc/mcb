//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#value-objects)
//!
//! Domain Type Definitions
//!
//! Type aliases and basic type definitions for dynamic domain concepts.
//! These allow the domain to be extended without changing core types.

use std::path::Path;

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Programming language identifier.
pub type Language = String;

/// Supported programming languages with compile-time safety.
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
    #[must_use]
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
            _ => None,
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
    #[must_use]
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

    /// Get all supported languages
    #[must_use]
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

/// System operation type identifier.
pub type OperationType = String;

/// Embedding provider identifier.
pub type EmbeddingProviderKind = String;

/// Vector store provider identifier.
pub type VectorStoreProviderKind = String;

/// Cache provider identifier.
pub type CacheProviderKind = String;
