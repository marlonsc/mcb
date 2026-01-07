//! Fallback chunking using regex patterns
//!
//! This module provides regex-based chunking as a fallback when tree-sitter
//! parsing is not available or fails.

use crate::chunking::config::LanguageConfig;
use crate::core::types::{CodeChunk, Language};
use regex;
use std::collections::HashMap;

/// Generic fallback chunker using regex patterns
pub struct GenericFallbackChunker<'a> {
    config: &'a LanguageConfig,
}

impl<'a> GenericFallbackChunker<'a> {
    pub fn new(config: &'a LanguageConfig) -> Self {
        Self { config }
    }

    pub fn chunk_with_patterns(
        &self,
        content: &str,
        file_name: &str,
        language: &Language,
    ) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_block = Vec::new();
        let mut block_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check if line matches any pattern
            let is_block_start = self.config.fallback_patterns.iter().any(|pattern| {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    regex.is_match(trimmed)
                } else {
                    false
                }
            });

            if is_block_start {
                if !current_block.is_empty() {
                    self.create_chunk(
                        &current_block,
                        block_start,
                        i - 1,
                        file_name,
                        language,
                        &mut chunks,
                    );
                    current_block.clear();
                }
                current_block.push(line.to_string());
                block_start = i;
            } else if !current_block.is_empty() {
                current_block.push(line.to_string());

                // Check for block end using brace counting
                if self.is_block_complete(&current_block) {
                    self.create_chunk(
                        &current_block,
                        block_start,
                        i,
                        file_name,
                        language,
                        &mut chunks,
                    );
                    current_block.clear();
                    block_start = i + 1;
                }
            }
        }

        if !current_block.is_empty() {
            self.create_chunk(
                &current_block,
                block_start,
                lines.len() - 1,
                file_name,
                language,
                &mut chunks,
            );
        }

        chunks
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

    fn create_chunk(
        &self,
        lines: &[String],
        start_line: usize,
        end_line: usize,
        file_name: &str,
        language: &Language,
        chunks: &mut Vec<CodeChunk>,
    ) {
        let content = lines.join("\n").trim().to_string();
        if content.is_empty() || content.len() < 20 {
            return;
        }

        let chunk = CodeChunk {
            id: format!("{}_{}_{}", file_name, start_line, end_line),
            content,
            file_path: file_name.to_string(),
            start_line: start_line as u32,
            end_line: end_line as u32,
            language: language.clone(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file".to_string(), serde_json::json!(file_name));
                meta.insert("chunk_type".to_string(), serde_json::json!("fallback"));
                serde_json::to_value(meta).unwrap_or(serde_json::json!({}))
            },
        };
        chunks.push(chunk);
    }
}
