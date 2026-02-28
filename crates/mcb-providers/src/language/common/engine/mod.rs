//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md)
//!
//! Intelligent chunking engine
//!
//! Provides the main `IntelligentChunker` that orchestrates language-specific
//! chunking using tree-sitter and fallback methods.

mod processors;
mod universal_provider;

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{ChunkingOptions, ChunkingResult, CodeChunker};
use mcb_domain::value_objects::Language;

use self::processors::LANGUAGE_PROCESSORS;
use super::constants::CHUNK_SIZE_GENERIC;
use super::detection::{is_language_supported, language_from_extension};
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
            mcb_domain::error!(
                "chunking_engine",
                "spawn_blocking panic in chunking engine",
                &e
            );
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
        file_path: &std::path::Path,
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
        file_paths: &[&std::path::Path],
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
pub use universal_provider::*;
