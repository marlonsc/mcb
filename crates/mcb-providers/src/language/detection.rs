//! Language Detection
//!
//! Provides functions to detect programming languages from file extensions,
//! check language support for AST chunking, and retrieve language-specific
//! configuration values.

use super::common::constants::{CHUNK_SIZE_GENERIC, EXTENSION_LANG_MAP, LANG_CHUNK_SIZE_MAP};
use mcb_domain::constants::lang::*;

/// Detect language from file extension
///
/// Returns a string identifier for the programming language based on the file extension.
/// Returns "unknown" for unsupported or unrecognized extensions.
pub fn language_from_extension(ext: &str) -> String {
    let ext_lower = ext.to_lowercase();
    EXTENSION_LANG_MAP
        .iter()
        .find(|(exts, _)| exts.iter().any(|e| *e == ext_lower))
        .map(|(_, lang)| (*lang).to_string())
        .unwrap_or_else(|| LANG_UNKNOWN.to_string())
}

/// Check if a language is supported for AST-based chunking
pub fn is_language_supported(language: &str) -> bool {
    matches!(
        language,
        LANG_RUST
            | LANG_PYTHON
            | LANG_JAVASCRIPT
            | LANG_TYPESCRIPT
            | LANG_GO
            | LANG_JAVA
            | LANG_C
            | LANG_CPP
            | LANG_CSHARP
            | LANG_RUBY
            | LANG_PHP
            | LANG_SWIFT
            | LANG_KOTLIN
    )
}

/// Get the chunk size for a specific language
pub fn get_chunk_size(language: &str) -> usize {
    LANG_CHUNK_SIZE_MAP
        .iter()
        .find(|(langs, _)| langs.contains(&language))
        .map(|(_, size)| *size)
        .unwrap_or(CHUNK_SIZE_GENERIC)
}

/// Get a list of all supported languages
pub fn supported_languages() -> Vec<String> {
    vec![
        LANG_RUST.to_string(),
        LANG_PYTHON.to_string(),
        LANG_JAVASCRIPT.to_string(),
        LANG_TYPESCRIPT.to_string(),
        LANG_GO.to_string(),
        LANG_JAVA.to_string(),
        LANG_C.to_string(),
        LANG_CPP.to_string(),
        LANG_CSHARP.to_string(),
        LANG_RUBY.to_string(),
        LANG_PHP.to_string(),
        LANG_SWIFT.to_string(),
        LANG_KOTLIN.to_string(),
    ]
}
