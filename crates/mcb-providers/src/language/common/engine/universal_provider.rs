//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Universal language chunking provider
//!
//! Provides a single provider that supports all languages by delegating to the `IntelligentChunker`.

use std::sync::Arc;

use mcb_domain::ports::LanguageChunkingProvider as LanguageProviderPort;
use mcb_domain::registry::language::{
    LANGUAGE_PROVIDERS, LanguageProviderConfig, LanguageProviderEntry,
};

use super::IntelligentChunker;

/// Universal Language Chunking Provider
///
/// A provider that supports all languages by delegating to the `IntelligentChunker`.
/// This is used for dependency injection where we need a single provider that
/// can handle any supported language.
pub struct UniversalLanguageChunkingProvider {
    chunker: IntelligentChunker,
}

impl UniversalLanguageChunkingProvider {
    /// Create a new universal language chunking provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            chunker: IntelligentChunker::new(),
        }
    }
}

impl Default for UniversalLanguageChunkingProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl mcb_domain::ports::LanguageChunkingProvider for UniversalLanguageChunkingProvider {
    fn language(&self) -> mcb_domain::value_objects::Language {
        "universal".to_owned()
    }

    fn extensions(&self) -> &[&'static str] {
        &[
            "rs", "py", "js", "ts", "java", "go", "c", "cpp", "cs", "rb", "php", "swift", "kt",
        ]
    }

    fn chunk(&self, content: &str, file_path: &str) -> Vec<mcb_domain::entities::CodeChunk> {
        let path = std::path::Path::new(file_path);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let language = super::super::detection::language_from_extension(ext);
        self.chunker.chunk_code(content, file_path, &language)
    }

    fn provider_name(&self) -> &str {
        "universal"
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

/// Factory function for creating universal language chunking provider instances.
fn universal_language_factory(
    _config: &LanguageProviderConfig,
) -> std::result::Result<Arc<dyn LanguageProviderPort>, String> {
    Ok(Arc::new(UniversalLanguageChunkingProvider::new()))
}

#[linkme::distributed_slice(LANGUAGE_PROVIDERS)]
#[allow(unsafe_code)]
static UNIVERSAL_LANGUAGE_PROVIDER: LanguageProviderEntry = LanguageProviderEntry {
    name: "universal",
    description: "Universal language chunker supporting all languages via tree-sitter",
    build: universal_language_factory,
};
