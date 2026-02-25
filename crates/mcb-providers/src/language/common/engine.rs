//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Intelligent chunking engine
//!
//! Provides the main `IntelligentChunker` that orchestrates language-specific
//! chunking using tree-sitter and fallback methods.

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use async_trait::async_trait;
use tracing;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{ChunkingOptions, ChunkingResult, CodeChunker};
use mcb_domain::value_objects::Language;

use super::constants::CHUNK_SIZE_GENERIC;
use super::detection::{is_language_supported, language_from_extension};
use crate::language::{
    CProcessor, CSharpProcessor, CppProcessor, GoProcessor, JavaProcessor, JavaScriptProcessor,
    KotlinProcessor, LanguageProcessor, PhpProcessor, PythonProcessor, RubyProcessor,
    RustProcessor, SwiftProcessor,
};

/// Language processor registry
pub(crate) static LANGUAGE_PROCESSORS: LazyLock<
    HashMap<String, Box<dyn LanguageProcessor + Send + Sync>>,
> = LazyLock::new(|| {
    let mut processors: HashMap<String, Box<dyn LanguageProcessor + Send + Sync>> = HashMap::new();

    processors.insert(
        "rust".to_owned(),
        Box::new(RustProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "python".to_owned(),
        Box::new(PythonProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "javascript".to_owned(),
        Box::new(JavaScriptProcessor::new(false)) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "typescript".to_owned(),
        Box::new(JavaScriptProcessor::new(true)) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "go".to_owned(),
        Box::new(GoProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "java".to_owned(),
        Box::new(JavaProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "c".to_owned(),
        Box::new(CProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "cpp".to_owned(),
        Box::new(CppProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "csharp".to_owned(),
        Box::new(CSharpProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "ruby".to_owned(),
        Box::new(RubyProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "php".to_owned(),
        Box::new(PhpProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "swift".to_owned(),
        Box::new(SwiftProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );
    processors.insert(
        "kotlin".to_owned(),
        Box::new(KotlinProcessor::new()) as Box<dyn LanguageProcessor + Send + Sync>,
    );

    processors
});

/// Intelligent chunking engine using tree-sitter
#[derive(Default)]
pub struct IntelligentChunker;

impl IntelligentChunker {
    /// Create a new intelligent chunker
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Chunk code based on language-specific structural analysis
    pub fn chunk_code(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        if let Some(processor) = LANGUAGE_PROCESSORS.get(language) {
            match Self::parse_with_tree_sitter(content, &processor.get_language()) {
                Ok(tree) => {
                    let chunks = processor
                        .extract_chunks_with_tree_sitter(&tree, content, file_name, language);
                    if !chunks.is_empty() {
                        return chunks;
                    }
                }
                Err(e) => {
                    mcb_domain::warn!(
                        "engine",
                        "tree-sitter parse failed, using generic chunking",
                        &format!("file = {file_name}, error = {e}")
                    );
                }
            }
        }

        Self::chunk_generic(content, file_name, language)
    }

    /// Chunk code asynchronously (offloads to blocking thread)
    pub async fn chunk_code_async(
        &self,
        content: String,
        file_name: String,
        language: Language,
    ) -> Vec<CodeChunk> {
        tokio::task::spawn_blocking(move || {
            let chunker = Self::new();
            chunker.chunk_code(&content, &file_name, &language)
        })
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = ?e, "spawn_blocking panic in chunking engine");
            Default::default()
        })
    }

    /// Generic chunking for unsupported languages
    fn chunk_generic(content: &str, file_name: &str, language: &Language) -> Vec<CodeChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let chunk_size = CHUNK_SIZE_GENERIC;
        // Clone language once before loop to avoid repeated allocations
        let lang = language.clone();
        let file = file_name.to_owned();

        for (chunk_idx, chunk_lines) in lines.chunks(chunk_size).enumerate() {
            let start_line = chunk_idx * chunk_size;
            let end_line = start_line + chunk_lines.len() - 1;

            let content = chunk_lines.join("\n").trim().to_owned();
            if content.is_empty() || content.len() < 20 {
                continue;
            }

            chunks.push(CodeChunk {
                id: format!("{file_name}_{chunk_idx}"),
                content,
                file_path: file.clone(),
                start_line: start_line as u32,
                end_line: end_line as u32,
                language: lang.clone(),
                metadata: serde_json::json!({
                    "file": file_name,
                    "chunk_index": chunk_idx,
                    "chunk_type": "generic"
                }),
            });
        }

        chunks
    }

    /// Parse code with tree-sitter
    fn parse_with_tree_sitter(
        content: &str,
        language: &tree_sitter::Language,
    ) -> Result<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(language)
            .map_err(|e| Error::internal(format!("Failed to set tree-sitter language: {e:?}")))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| Error::internal("Tree-sitter parsing failed".to_owned()))?;

        Ok(tree)
    }
}

#[async_trait]
impl CodeChunker for IntelligentChunker {
    async fn chunk_file(
        &self,
        file_path: &Path,
        _options: ChunkingOptions,
    ) -> Result<ChunkingResult> {
        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| Error::io(e.to_string()))?;

        let file_name = mcb_domain::utils::path::path_to_utf8_string(file_path)
            .map_err(|e| Error::io(e.to_string()))?;
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let language = language_from_extension(ext);

        self.chunk_content(&content, &file_name, language, _options)
            .await
    }

    async fn chunk_content(
        &self,
        content: &str,
        file_name: &str,
        language: Language,
        _options: ChunkingOptions,
    ) -> Result<ChunkingResult> {
        let chunks = self.chunk_code(content, file_name, &language);
        let used_ast = is_language_supported(&language);

        Ok(ChunkingResult {
            file_path: file_name.to_owned(),
            language,
            chunks,
            used_ast,
        })
    }

    async fn chunk_batch(
        &self,
        file_paths: &[&Path],
        options: ChunkingOptions,
    ) -> Result<Vec<ChunkingResult>> {
        let mut results = Vec::with_capacity(file_paths.len());
        for path in file_paths {
            results.push(self.chunk_file(path, options).await?);
        }
        Ok(results)
    }

    fn supported_languages(&self) -> Vec<Language> {
        LANGUAGE_PROCESSORS.keys().cloned().collect()
    }
}

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
        let language = super::detection::language_from_extension(ext);
        self.chunker.chunk_code(content, file_path, &language)
    }

    fn provider_name(&self) -> &str {
        "universal"
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::ports::LanguageChunkingProvider as LanguageProviderPort;
use mcb_domain::registry::language::{
    LANGUAGE_PROVIDERS, LanguageProviderConfig, LanguageProviderEntry,
};

/// Factory function for creating universal language chunking provider instances.
fn universal_language_factory(
    _config: &LanguageProviderConfig,
) -> std::result::Result<Arc<dyn LanguageProviderPort>, String> {
    Ok(Arc::new(UniversalLanguageChunkingProvider::new()))
}

#[linkme::distributed_slice(LANGUAGE_PROVIDERS)]
static UNIVERSAL_LANGUAGE_PROVIDER: LanguageProviderEntry = LanguageProviderEntry {
    name: "universal",
    description: "Universal language chunker supporting all languages via tree-sitter",
    build: universal_language_factory,
};
