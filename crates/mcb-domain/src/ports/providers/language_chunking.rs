//! Language-specific chunking provider ports.

use crate::entities::CodeChunk;
use crate::value_objects::Language;

/// Language-Specific Code Chunking Provider.
pub trait LanguageChunkingProvider: Send + Sync {
    /// Get the primary language supported by this chunker.
    fn language(&self) -> Language;
    /// Get the list of file extensions this provider can process.
    fn extensions(&self) -> &[&'static str];
    /// Split source code into semantically relevant chunks.
    fn chunk(&self, content: &str, file_path: &str) -> Vec<CodeChunk>;
    /// Get the unique name of this chunking implementation.
    fn provider_name(&self) -> &str;

    /// Check if this provider supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
    }

    /// Get the recommended maximum size for chunks in tokens or characters.
    fn max_chunk_size(&self) -> usize {
        50
    }
}
