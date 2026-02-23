//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Language Detection
//!
//! Provides functions to detect programming languages from file extensions,
//! check language support for AST chunking, and retrieve language-specific
//! configuration values.

use super::constants::{CHUNK_SIZE_GENERIC, EXTENSION_LANG_MAP, LANG_CHUNK_SIZE_MAP};
use mcb_domain::constants::lang::*;

/// Detect language from file extension
///
/// Returns a string identifier for the programming language based on the file extension.
/// Returns "unknown" for unsupported or unrecognized extensions.
#[must_use]
pub fn language_from_extension(ext: &str) -> String {
    let ext_lower = ext.to_lowercase();
    EXTENSION_LANG_MAP
        .iter()
        .find(|(exts, _)| exts.iter().any(|e| *e == ext_lower))
        .map_or_else(|| LANG_UNKNOWN.to_owned(), |(_, lang)| (*lang).to_owned())
}

/// Check if a language is supported for AST-based chunking
#[must_use]
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
#[must_use]
pub fn get_chunk_size(language: &str) -> usize {
    LANG_CHUNK_SIZE_MAP
        .iter()
        .find(|(langs, _)| langs.contains(&language))
        .map_or(CHUNK_SIZE_GENERIC, |(_, size)| *size)
}

/// Get a list of all supported languages
#[must_use]
pub fn supported_languages() -> Vec<String> {
    vec![
        LANG_RUST.to_owned(),
        LANG_PYTHON.to_owned(),
        LANG_JAVASCRIPT.to_owned(),
        LANG_TYPESCRIPT.to_owned(),
        LANG_GO.to_owned(),
        LANG_JAVA.to_owned(),
        LANG_C.to_owned(),
        LANG_CPP.to_owned(),
        LANG_CSHARP.to_owned(),
        LANG_RUBY.to_owned(),
        LANG_PHP.to_owned(),
        LANG_SWIFT.to_owned(),
        LANG_KOTLIN.to_owned(),
    ]
}
