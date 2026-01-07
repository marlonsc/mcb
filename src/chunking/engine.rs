//! Intelligent chunking engine
//!
//! This module provides the main IntelligentChunker that orchestrates
//! language-specific chunking using tree-sitter and fallback methods.

use crate::core::error::Result;
use crate::core::types::{CodeChunk, Language};
use std::collections::HashMap;

/// Intelligent chunking engine using tree-sitter
#[derive(Default)]
pub struct IntelligentChunker;

impl IntelligentChunker {
    /// Create a new intelligent chunker
    pub fn new() -> Self {
        Self
    }

    /// Check if a language is supported for intelligent chunking
    pub fn is_language_supported(language: &Language) -> bool {
        crate::chunking::LANGUAGE_CONFIGS.contains_key(language)
    }

    /// Chunk code based on language-specific structural analysis
    pub fn chunk_code(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        if let Some(processor) = crate::chunking::LANGUAGE_CONFIGS.get(&language) {
            // Try tree-sitter parsing first
            match self.parse_with_tree_sitter(content, processor.get_language()) {
                Ok(tree) => {
                    let chunks = processor
                        .extract_chunks_with_tree_sitter(&tree, content, file_name, &language);
                    if !chunks.is_empty() {
                        return chunks;
                    }
                }
                Err(_) => {
                    // Fall back to pattern-based chunking
                    let chunks = processor.extract_chunks_fallback(content, file_name, &language);
                    if !chunks.is_empty() {
                        return chunks;
                    }
                }
            }
        }

        // Ultimate fallback to generic chunking
        self.chunk_generic(content, file_name, language)
    }

    /// Generic chunking for unsupported languages
    fn chunk_generic(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let chunk_size = 15; // lines per chunk for generic code

        for (chunk_idx, chunk_lines) in lines.chunks(chunk_size).enumerate() {
            let start_line = chunk_idx * chunk_size;
            let end_line = start_line + chunk_lines.len() - 1;

            let content = chunk_lines.join("\n").trim().to_string();
            if content.is_empty() || content.len() < 20 {
                continue;
            }

            let chunk = CodeChunk {
                id: format!("{}_{}", file_name, chunk_idx),
                content,
                file_path: file_name.to_string(),
                start_line: start_line as u32,
                end_line: end_line as u32,
                language: language.clone(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("file".to_string(), serde_json::json!(file_name));
                    meta.insert("chunk_index".to_string(), serde_json::json!(chunk_idx));
                    meta.insert("chunk_type".to_string(), serde_json::json!("generic"));
                    serde_json::to_value(meta).unwrap_or(serde_json::json!({}))
                },
            };
            chunks.push(chunk);
        }

        chunks
    }

    /// Parse code with tree-sitter
    fn parse_with_tree_sitter(
        &self,
        content: &str,
        language: tree_sitter::Language,
    ) -> Result<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).map_err(|e| {
            crate::core::error::Error::internal(format!(
                "Failed to set tree-sitter language: {:?}",
                e
            ))
        })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            crate::core::error::Error::internal("Tree-sitter parsing failed".to_string())
        })?;

        Ok(tree)
    }
}
