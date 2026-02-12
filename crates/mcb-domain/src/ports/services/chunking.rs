//! Provides chunking domain definitions.
use std::path::Path;

use async_trait::async_trait;

use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::Language;

/// Options for chunking operations
#[derive(Debug, Clone, Copy)]
pub struct ChunkingOptions {
    /// Maximum size of a single chunk in characters
    pub max_chunk_size: usize,
    /// Whether to include surrounding context (imports, class declarations, etc.)
    pub include_context: bool,
    /// Maximum number of chunks per file
    pub max_chunks_per_file: usize,
}

impl Default for ChunkingOptions {
    fn default() -> Self {
        Self {
            max_chunk_size: 512,
            include_context: true,
            max_chunks_per_file: 50,
        }
    }
}

/// Result of chunking a single file
#[derive(Debug, Clone)]
pub struct ChunkingResult {
    /// File path that was chunked
    pub file_path: String,
    /// Language detected for the file
    pub language: Language,
    /// Extracted chunks
    pub chunks: Vec<CodeChunk>,
    /// Whether AST parsing was successful (vs fallback)
    pub used_ast: bool,
}

/// Chunking Orchestrator Interface
///
/// Coordinates batch code chunking operations.
#[async_trait]
pub trait ChunkingOrchestratorInterface: Send + Sync {
    /// Process multiple files and return chunks
    async fn process_files(&self, files: Vec<(String, String)>) -> Result<Vec<CodeChunk>>;

    /// Process a single file with content
    async fn process_file(&self, path: &Path, content: &str) -> Result<Vec<CodeChunk>>;

    /// Read and chunk a file from disk
    async fn chunk_file(&self, path: &Path) -> Result<Vec<CodeChunk>>;
}

/// Domain Port for Code Chunking Operations
#[async_trait]
pub trait CodeChunker: Send + Sync {
    /// Performs the chunk file operation.
    async fn chunk_file(
        &self,
        file_path: &Path,
        options: ChunkingOptions,
    ) -> Result<ChunkingResult>;

    /// Performs the chunk content operation.
    async fn chunk_content(
        &self,
        content: &str,
        file_name: &str,
        language: Language,
        options: ChunkingOptions,
    ) -> Result<ChunkingResult>;

    /// Performs the chunk batch operation.
    async fn chunk_batch(
        &self,
        file_paths: &[&Path],
        options: ChunkingOptions,
    ) -> Result<Vec<ChunkingResult>>;

    /// Performs the supported languages operation.
    fn supported_languages(&self) -> Vec<Language>;

    /// Performs the is language supported operation.
    fn is_language_supported(&self, language: &Language) -> bool {
        self.supported_languages().contains(language)
    }
}
