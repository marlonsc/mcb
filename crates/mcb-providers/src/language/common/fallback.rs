//! Fallback chunking using regex patterns
//!
//! This module provides regex-based chunking as a fallback when tree-sitter
//! parsing is not available or fails.

use super::config::LanguageConfig;
use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::Language;
use regex::Regex;
use std::collections::HashMap;

/// Parameters for creating a code chunk
#[derive(Debug)]
pub struct ChunkCreationParams<'a> {
    /// Lines of code to include in the chunk
    pub lines: &'a [String],
    /// Starting line number
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
    /// Source file name
    pub file_name: &'a str,
    /// Programming language
    pub language: &'a Language,
}

/// Context for chunking operations
struct ChunkingContext<'a> {
    lines: &'a [&'a str],
    chunks: &'a mut Vec<CodeChunk>,
    current_block: &'a mut Vec<String>,
    block_start: &'a mut usize,
    file_name: &'a str,
    language: &'a Language,
}

/// Generic fallback chunker using regex patterns
pub struct GenericFallbackChunker<'a> {
    #[allow(dead_code)] // Reserved for future config-based pattern selection
    config: &'a LanguageConfig,
    /// Precompiled regex patterns for block detection
    compiled_patterns: Vec<Regex>,
}

impl<'a> GenericFallbackChunker<'a> {
    /// Create a new generic fallback chunker with language configuration
    pub fn new(config: &'a LanguageConfig) -> Self {
        let compiled_patterns = config
            .fallback_patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        Self {
            config,
            compiled_patterns,
        }
    }

    /// Chunk content using regex patterns as a fallback
    pub fn chunk_with_patterns(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_block = Vec::new();
        let mut block_start = 0;

        let mut context = ChunkingContext {
            lines: &lines,
            chunks: &mut chunks,
            current_block: &mut current_block,
            block_start: &mut block_start,
            file_name,
            language,
        };

        self.process_lines(&mut context);
        self.finalize_remaining_block(&mut context);

        chunks
    }

    /// Process all lines to identify and create chunks
    fn process_lines(&self, context: &mut ChunkingContext) {
        for (i, line) in context.lines.iter().enumerate() {
            let is_block_start = self.is_pattern_match(line.trim());

            if is_block_start && !context.current_block.is_empty() {
                self.create_chunk_from_block(context, *context.block_start, i - 1);
                context.current_block.clear();
            }

            if is_block_start {
                context.current_block.push(line.to_string());
                *context.block_start = i;
            } else if !context.current_block.is_empty() {
                context.current_block.push(line.to_string());
                if self.is_block_complete(context.current_block) {
                    self.create_chunk_from_block(context, *context.block_start, i);
                    context.current_block.clear();
                    *context.block_start = i + 1;
                }
            }
        }
    }

    /// Create a chunk from a completed block
    fn create_chunk_from_block(
        &self,
        context: &mut ChunkingContext,
        start_line: usize,
        end_line: usize,
    ) {
        let params = ChunkCreationParams {
            lines: context.current_block,
            start_line,
            end_line,
            file_name: context.file_name,
            language: context.language,
        };
        self.create_chunk(&params, context.chunks);
    }

    /// Finalize any remaining block at the end
    fn finalize_remaining_block(&self, context: &mut ChunkingContext) {
        if !context.current_block.is_empty() {
            self.create_chunk_from_block(context, *context.block_start, context.lines.len() - 1);
        }
    }

    fn is_pattern_match(&self, line: &str) -> bool {
        self.compiled_patterns
            .iter()
            .any(|regex| regex.is_match(line))
    }

    fn is_block_complete(&self, block: &[String]) -> bool {
        let open_count: usize = block
            .iter()
            .map(|line| line.chars().filter(|&c| c == '{').count())
            .sum();
        let close_count: usize = block
            .iter()
            .map(|line| line.chars().filter(|&c| c == '}').count())
            .sum();

        open_count > 0 && open_count == close_count && block.len() > 2
    }

    fn create_chunk(&self, params: &ChunkCreationParams, chunks: &mut Vec<CodeChunk>) {
        let content = params.lines.join("\n").trim().to_string();
        if content.is_empty() || content.len() < 20 {
            return;
        }

        let chunk = CodeChunk {
            id: format!(
                "{}_{}_{}",
                params.file_name, params.start_line, params.end_line
            ),
            content,
            file_path: params.file_name.to_string(),
            start_line: params.start_line as u32,
            end_line: params.end_line as u32,
            language: params.language.clone(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file".to_string(), serde_json::json!(params.file_name));
                meta.insert("chunk_type".to_string(), serde_json::json!("fallback"));
                serde_json::to_value(meta).unwrap_or(serde_json::json!({}))
            },
        };
        chunks.push(chunk);
    }
}
