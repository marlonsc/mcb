//! Language-Specific Code Chunking
//!
//! Provides chunking strategies for breaking source code into semantic units.
//! Different languages have different optimal chunking strategies.

use async_trait::async_trait;
use std::path::Path;

use crate::error::Result;
use crate::language::LanguageId;
use crate::parser::{Parser, RcaParser};

/// A chunk of code with metadata
#[derive(Debug, Clone)]
pub struct CodeChunk {
    /// The chunk content
    pub content: String,
    /// Start line (1-indexed)
    pub start_line: usize,
    /// End line (1-indexed)
    pub end_line: usize,
    /// Chunk type (e.g., "function", "class", "module")
    pub chunk_type: ChunkType,
    /// Optional name (e.g., function name)
    pub name: Option<String>,
}

/// Type of code chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkType {
    /// A complete function or method
    Function,
    /// A class or struct definition
    Class,
    /// A module or file-level code
    Module,
    /// An arbitrary block of code (fallback)
    Block,
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkType::Function => write!(f, "function"),
            ChunkType::Class => write!(f, "class"),
            ChunkType::Module => write!(f, "module"),
            ChunkType::Block => write!(f, "block"),
        }
    }
}

/// Configuration for chunking strategy
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    /// Maximum lines per chunk
    pub max_lines: usize,
    /// Minimum lines per chunk (avoid tiny chunks)
    pub min_lines: usize,
    /// Target lines per chunk (soft limit)
    pub target_lines: usize,
    /// Include surrounding context lines
    pub context_lines: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_lines: 100,
            min_lines: 5,
            target_lines: 50,
            context_lines: 3,
        }
    }
}

/// Chunking strategy trait for language-specific implementations
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    /// Chunk source code into semantic units
    async fn chunk(
        &self,
        content: &str,
        language: LanguageId,
        path: &Path,
    ) -> Result<Vec<CodeChunk>>;

    /// Get the configuration for this strategy
    fn config(&self) -> &ChunkingConfig;
}

/// Semantic chunking strategy using AST analysis
///
/// Chunks code at function/method boundaries for better semantic coherence.
pub struct SemanticChunking {
    config: ChunkingConfig,
    parser: RcaParser,
}

impl Default for SemanticChunking {
    fn default() -> Self {
        Self::new(ChunkingConfig::default())
    }
}

impl SemanticChunking {
    /// Create new semantic chunking strategy
    pub fn new(config: ChunkingConfig) -> Self {
        Self {
            config,
            parser: RcaParser::new(),
        }
    }

    /// Create with custom parser
    pub fn with_parser(config: ChunkingConfig, parser: RcaParser) -> Self {
        Self { config, parser }
    }

    /// Extract lines from content
    fn extract_lines(content: &str, start: usize, end: usize) -> &str {
        let lines: Vec<&str> = content.lines().collect();
        let start_idx = start.saturating_sub(1);
        let end_idx = end.min(lines.len());

        if start_idx >= lines.len() || start_idx >= end_idx {
            return "";
        }

        // Find byte positions
        let mut byte_start = 0;
        for (i, line) in content.lines().enumerate() {
            if i == start_idx {
                break;
            }
            byte_start += line.len() + 1; // +1 for newline
        }

        let mut byte_end = byte_start;
        for (i, line) in content.lines().enumerate().skip(start_idx) {
            if i >= end_idx {
                break;
            }
            byte_end += line.len() + 1;
        }

        // Handle edge case where content doesn't end with newline
        byte_end = byte_end.min(content.len());

        &content[byte_start..byte_end]
    }
}

#[async_trait]
impl ChunkingStrategy for SemanticChunking {
    async fn chunk(
        &self,
        content: &str,
        language: LanguageId,
        path: &Path,
    ) -> Result<Vec<CodeChunk>> {
        let parsed = self
            .parser
            .parse_content(content.as_bytes(), language, path)
            .await?;

        let mut chunks = Vec::new();

        // Create chunks from functions
        for func in &parsed.functions {
            let chunk_content = Self::extract_lines(content, func.start_line, func.end_line);
            let line_count = func.end_line - func.start_line + 1;

            // Skip very small functions, they'll be included in module chunk
            if line_count < self.config.min_lines {
                continue;
            }

            // If function is too large, we still include it as one semantic unit
            // (breaking a function mid-way loses semantic coherence)
            chunks.push(CodeChunk {
                content: chunk_content.to_string(),
                start_line: func.start_line,
                end_line: func.end_line,
                chunk_type: ChunkType::Function,
                name: Some(func.name.clone()),
            });
        }

        // If no functions found or very small file, treat whole file as one chunk
        if chunks.is_empty() {
            let lines: Vec<&str> = content.lines().collect();
            if !lines.is_empty() {
                chunks.push(CodeChunk {
                    content: content.to_string(),
                    start_line: 1,
                    end_line: lines.len(),
                    chunk_type: ChunkType::Module,
                    name: path.file_name().map(|n| n.to_string_lossy().to_string()),
                });
            }
        }

        // Sort chunks by start line
        chunks.sort_by_key(|c| c.start_line);

        Ok(chunks)
    }

    fn config(&self) -> &ChunkingConfig {
        &self.config
    }
}

/// Simple line-based chunking (fallback strategy)
///
/// Chunks code by line count, useful when AST parsing fails.
pub struct LineBasedChunking {
    config: ChunkingConfig,
}

impl Default for LineBasedChunking {
    fn default() -> Self {
        Self::new(ChunkingConfig::default())
    }
}

impl LineBasedChunking {
    /// Create new line-based chunking strategy
    pub fn new(config: ChunkingConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ChunkingStrategy for LineBasedChunking {
    async fn chunk(
        &self,
        content: &str,
        _language: LanguageId,
        path: &Path,
    ) -> Result<Vec<CodeChunk>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();

        if lines.is_empty() {
            return Ok(chunks);
        }

        let chunk_size = self.config.target_lines;
        let mut start = 0;

        while start < lines.len() {
            let end = (start + chunk_size).min(lines.len());
            let chunk_lines = &lines[start..end];
            let chunk_content = chunk_lines.join("\n");

            chunks.push(CodeChunk {
                content: chunk_content,
                start_line: start + 1, // 1-indexed
                end_line: end,
                chunk_type: ChunkType::Block,
                name: if start == 0 {
                    path.file_name().map(|n| n.to_string_lossy().to_string())
                } else {
                    None
                },
            });

            start = end;
        }

        Ok(chunks)
    }

    fn config(&self) -> &ChunkingConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_chunking_rust() {
        let chunker = SemanticChunking::default();
        let code = r#"fn foo() {
    println!("foo");
}

fn bar(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else {
        -x
    }
}
"#;

        let chunks = chunker
            .chunk(code, LanguageId::Rust, Path::new("test.rs"))
            .await
            .expect("Should chunk");

        // Should find the functions
        assert!(!chunks.is_empty());

        // All chunks should have valid line numbers
        for chunk in &chunks {
            assert!(chunk.start_line >= 1);
            assert!(chunk.end_line >= chunk.start_line);
        }
    }

    #[tokio::test]
    async fn test_semantic_chunking_small_file() {
        let chunker = SemanticChunking::default();
        let code = "x = 1\ny = 2\n";

        let chunks = chunker
            .chunk(code, LanguageId::Python, Path::new("test.py"))
            .await
            .expect("Should chunk");

        // Small file should be treated as single module chunk
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::Module);
    }

    #[tokio::test]
    async fn test_line_based_chunking() {
        let config = ChunkingConfig {
            target_lines: 3,
            ..Default::default()
        };
        let chunker = LineBasedChunking::new(config);

        let code = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n";

        let chunks = chunker
            .chunk(code, LanguageId::Python, Path::new("test.py"))
            .await
            .expect("Should chunk");

        // Should create multiple chunks of ~3 lines each
        assert!(chunks.len() >= 2);

        for chunk in &chunks {
            assert_eq!(chunk.chunk_type, ChunkType::Block);
        }
    }

    #[tokio::test]
    async fn test_empty_content() {
        let chunker = SemanticChunking::default();
        let chunks = chunker
            .chunk("", LanguageId::Rust, Path::new("empty.rs"))
            .await
            .expect("Should handle empty");

        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_type_display() {
        assert_eq!(ChunkType::Function.to_string(), "function");
        assert_eq!(ChunkType::Class.to_string(), "class");
        assert_eq!(ChunkType::Module.to_string(), "module");
        assert_eq!(ChunkType::Block.to_string(), "block");
    }
}
