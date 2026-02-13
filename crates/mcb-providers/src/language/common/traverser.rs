//! AST traverser for extracting code chunks based on rules
//!
//! This module provides the AstTraverser that walks tree-sitter ASTs
//! and extracts code chunks according to configurable rules.

use mcb_domain::entities::CodeChunk;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::Language;

use super::config::NodeExtractionRule;

/// Parameters for creating a code chunk
#[derive(Debug)]
struct ChunkParams<'a> {
    content: String,
    file_name: &'a str,
    node_type: &'a str,
    depth: usize,
    priority: i32,
    chunk_index: usize,
}

/// Generic AST node traverser with configurable rules
pub struct AstTraverser<'a> {
    rules: &'a [NodeExtractionRule],
    language: &'a Language,
    max_chunks: usize,
}

impl<'a> AstTraverser<'a> {
    /// Create a new AST traverser with extraction rules and language configuration
    pub fn new(rules: &'a [NodeExtractionRule], language: &'a Language) -> Self {
        Self {
            rules,
            language,
            max_chunks: 100,
        }
    }

    /// Configure the maximum number of chunks to extract
    pub fn with_max_chunks(mut self, max_chunks: usize) -> Self {
        self.max_chunks = max_chunks;
        self
    }

    fn rule_matches_node_type(rule: &NodeExtractionRule, node_type: &str) -> bool {
        rule.node_types
            .iter()
            .any(|candidate| candidate == node_type)
    }

    fn can_descend(&self, depth: usize) -> bool {
        self.rules.iter().any(|rule| depth < rule.max_depth)
    }

    /// Traverse the AST and extract code chunks according to the configured rules
    // TODO(qlty): Function with high complexity (count = 29): traverse_and_extract
    pub fn traverse_and_extract(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        file_name: &str,
        depth: usize,
        chunks: &mut Vec<CodeChunk>,
    ) {
        // Stop if we've reached the chunk limit
        if chunks.len() >= self.max_chunks {
            return;
        }

        loop {
            let node = cursor.node();
            let node_type = node.kind();

            // Check if this node matches any extraction rule
            for rule in self.rules {
                if !Self::rule_matches_node_type(rule, node_type) {
                    continue;
                }
                if let Some(chunk) =
                    self.try_extract_chunk(node, content, file_name, depth, rule, chunks.len())
                {
                    chunks.push(chunk);
                    if chunks.len() >= self.max_chunks {
                        return;
                    }
                }
            }

            // Recurse into children if within depth limit
            if self.can_descend(depth) && cursor.goto_first_child() {
                self.traverse_and_extract(cursor, content, file_name, depth + 1, chunks);

                if chunks.len() >= self.max_chunks {
                    return;
                }

                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    /// Extracts the text content of a node.
    fn extract_node_content(node: tree_sitter::Node, content: &str) -> Result<String> {
        let Some(code) = Self::extract_node_slice(node, content) else {
            return Err(Error::internal("Invalid node range".to_string()));
        };
        if code.is_empty() {
            return Err(Error::internal("Empty node content".to_string()));
        }

        Ok(code.to_string())
    }

    fn extract_node_slice<'b>(node: tree_sitter::Node, content: &'b str) -> Option<&'b str> {
        let start = node.start_byte();
        let end = node.end_byte();
        if start >= content.len() || end > content.len() || start >= end {
            return None;
        }
        Some(content[start..end].trim())
    }

    /// Extracts node content with surrounding context lines.
    fn extract_node_with_context(
        node: tree_sitter::Node,
        content: &str,
        context_lines: usize,
    ) -> (Option<String>, Option<usize>) {
        let Some(code) = Self::extract_node_slice(node, content) else {
            return (None, None);
        };

        let start = node.start_byte();

        let lines: Vec<&str> = content.lines().collect();
        let start_line = content[..start].lines().count();
        let end_line = start_line + code.lines().count() - 1;

        let context_start = start_line.saturating_sub(context_lines);
        let context_end = (end_line + context_lines).min(lines.len());

        let context = lines[context_start..context_end].join("\n");

        (Some(context), Some(context_lines))
    }

    /// Try to extract a chunk from a node matching a rule
    fn try_extract_chunk(
        &self,
        node: tree_sitter::Node,
        content: &str,
        file_name: &str,
        depth: usize,
        rule: &NodeExtractionRule,
        chunk_index: usize,
    ) -> Option<CodeChunk> {
        let (code, context) = if rule.include_context {
            Self::extract_node_with_context(node, content, 3)
        } else {
            (Self::extract_node_content(node, content).ok(), None)
        };

        let code = code?;
        if code.len() < rule.min_length || code.lines().count() < rule.min_lines {
            return None;
        }

        let chunk_params = ChunkParams {
            content: code,
            file_name,
            node_type: node.kind(),
            depth,
            priority: rule.priority,
            chunk_index,
        };
        let mut chunk = self.create_chunk_from_node(node, chunk_params);

        // Add context metadata if available
        if let Some(context_lines) = context
            && let Some(metadata) = chunk.metadata.as_object_mut()
        {
            metadata.insert(
                "context_lines".to_string(),
                serde_json::json!(context_lines),
            );
        }

        Some(chunk)
    }

    fn create_chunk_from_node(&self, node: tree_sitter::Node, params: ChunkParams) -> CodeChunk {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;

        CodeChunk {
            id: format!(
                "{}_{}_{}_{}_{}_{}",
                params.file_name,
                params.node_type,
                start_line,
                end_line,
                params.priority,
                params.chunk_index
            ),
            content: params.content,
            file_path: params.file_name.to_string(),
            start_line: start_line as u32,
            end_line: end_line as u32,
            language: self.language.clone(),
            metadata: serde_json::json!({
                "file": params.file_name,
                "node_type": params.node_type,
                "depth": params.depth,
                "priority": params.priority,
            }),
        }
    }
}
