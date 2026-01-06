//! Intelligent code chunking using tree-sitter for structural parsing
//!
//! Provides language-aware chunking that respects code structure rather than
//! naive line-based or character-based splitting.

use crate::core::error::{Error, Result};
use crate::core::types::{CodeChunk, Language};
use std::collections::HashMap;

/// Intelligent chunking engine using tree-sitter
pub struct IntelligentChunker;

impl IntelligentChunker {
    /// Create a new intelligent chunker
    pub fn new() -> Self {
        Self
    }

    /// Chunk code based on language-specific structural analysis
    pub fn chunk_code(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        match language {
            Language::Rust => self.chunk_rust(content, file_name),
            Language::Python => self.chunk_python(content, file_name),
            Language::JavaScript | Language::TypeScript => self.chunk_javascript(content, file_name, language),
            _ => self.chunk_generic(content, file_name, language),
        }
    }

    /// Chunk Rust code using tree-sitter
    fn chunk_rust(&self, content: &str, file_name: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();

        // Try tree-sitter parsing first
        match self.parse_with_tree_sitter(content, tree_sitter_rust::language()) {
            Ok(tree) => {
                self.extract_rust_nodes(&tree, content, file_name, &mut chunks);
            }
            Err(_) => {
                // Fallback to regex-based parsing
                self.chunk_rust_fallback(content, file_name)
            }
        }

        if chunks.is_empty() {
            self.chunk_generic(content, file_name, Language::Rust)
        } else {
            chunks
        }
    }

    /// Chunk Python code using tree-sitter
    fn chunk_python(&self, content: &str, file_name: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();

        match self.parse_with_tree_sitter(content, tree_sitter_python::language()) {
            Ok(tree) => {
                self.extract_python_nodes(&tree, content, file_name, &mut chunks);
            }
            Err(_) => {
                self.chunk_python_fallback(content, file_name)
            }
        }

        if chunks.is_empty() {
            self.chunk_generic(content, file_name, Language::Python)
        } else {
            chunks
        }
    }

    /// Chunk JavaScript/TypeScript code using tree-sitter
    fn chunk_javascript(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let ts_language = match language {
            Language::TypeScript => tree_sitter_typescript::language_tsx(),
            _ => tree_sitter_javascript::language(),
        };

        match self.parse_with_tree_sitter(content, ts_language) {
            Ok(tree) => {
                self.extract_javascript_nodes(&tree, content, file_name, &mut chunks);
            }
            Err(_) => {
                self.chunk_javascript_fallback(content, file_name, language)
            }
        }

        if chunks.is_empty() {
            self.chunk_generic(content, file_name, language)
        } else {
            chunks
        }
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
                language,
                embedding: None,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("file".to_string(), serde_json::json!(file_name));
                    meta.insert("chunk_index".to_string(), serde_json::json!(chunk_idx));
                    meta.insert("chunk_type".to_string(), serde_json::json!("generic"));
                    meta
                },
            };
            chunks.push(chunk);
        }

        chunks
    }

    /// Parse code with tree-sitter
    fn parse_with_tree_sitter(&self, content: &str, language: tree_sitter::Language) -> Result<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language)
            .map_err(|e| Error::internal(format!("Failed to set tree-sitter language: {:?}", e)))?;

        let tree = parser.parse(content, None)
            .ok_or_else(|| Error::internal("Tree-sitter parsing failed".to_string()))?;

        Ok(tree)
    }

    /// Extract meaningful nodes from Rust AST
    fn extract_rust_nodes(&self, tree: &tree_sitter::Tree, content: &str, file_name: &str, chunks: &mut Vec<CodeChunk>) {
        let mut cursor = tree.walk();

        // Traverse the tree to find functions, structs, impls, etc.
        if cursor.goto_first_child() {
            self.traverse_rust_node(&mut cursor, content, file_name, 0, chunks);
        }
    }

    /// Recursively traverse Rust AST nodes
    fn traverse_rust_node(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        file_name: &str,
        depth: usize,
        chunks: &mut Vec<CodeChunk>
    ) {
        loop {
            let node = cursor.node();
            let node_type = node.kind();

            // Extract meaningful code blocks
            if matches!(node_type, "function_item" | "struct_item" | "impl_item" | "trait_item" | "mod_item") {
                if let Ok(code) = self.extract_node_content(node, content) {
                    if code.len() > 50 && code.lines().count() > 2 {
                        let start_line = node.start_position().row;
                        let end_line = node.end_position().row;

                        let chunk = CodeChunk {
                            id: format!("{}_{}_{}", file_name, node_type, start_line),
                            content: code,
                            file_path: file_name.to_string(),
                            start_line: start_line as u32,
                            end_line: end_line as u32,
                            language: Language::Rust,
                            embedding: None,
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert("file".to_string(), serde_json::json!(file_name));
                                meta.insert("node_type".to_string(), serde_json::json!(node_type));
                                meta.insert("depth".to_string(), serde_json::json!(depth));
                                meta
                            },
                        };
                        chunks.push(chunk);
                    }
                }
            }

            // Recurse into children (but limit depth to avoid too many chunks)
            if depth < 3 && cursor.goto_first_child() {
                self.traverse_rust_node(cursor, content, file_name, depth + 1, chunks);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    /// Extract Python nodes (functions, classes)
    fn extract_python_nodes(&self, tree: &tree_sitter::Tree, content: &str, file_name: &str, chunks: &mut Vec<CodeChunk>) {
        let mut cursor = tree.walk();

        if cursor.goto_first_child() {
            self.traverse_python_node(&mut cursor, content, file_name, 0, chunks);
        }
    }

    fn traverse_python_node(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        file_name: &str,
        depth: usize,
        chunks: &mut Vec<CodeChunk>
    ) {
        loop {
            let node = cursor.node();
            let node_type = node.kind();

            if matches!(node_type, "function_definition" | "class_definition") {
                if let Ok(code) = self.extract_node_content(node, content) {
                    if code.len() > 30 {
                        let start_line = node.start_position().row;
                        let end_line = node.end_position().row;

                        let chunk = CodeChunk {
                            id: format!("{}_{}_{}", file_name, node_type, start_line),
                            content: code,
                            file_path: file_name.to_string(),
                            start_line: start_line as u32,
                            end_line: end_line as u32,
                            language: Language::Python,
                            embedding: None,
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert("file".to_string(), serde_json::json!(file_name));
                                meta.insert("node_type".to_string(), serde_json::json!(node_type));
                                meta.insert("depth".to_string(), serde_json::json!(depth));
                                meta
                            },
                        };
                        chunks.push(chunk);
                    }
                }
            }

            if depth < 2 && cursor.goto_first_child() {
                self.traverse_python_node(cursor, content, file_name, depth + 1, chunks);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    /// Extract JavaScript/TypeScript nodes
    fn extract_javascript_nodes(&self, tree: &tree_sitter::Tree, content: &str, file_name: &str, chunks: &mut Vec<CodeChunk>) {
        let mut cursor = tree.walk();

        if cursor.goto_first_child() {
            self.traverse_javascript_node(&mut cursor, content, file_name, 0, chunks);
        }
    }

    fn traverse_javascript_node(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        content: &str,
        file_name: &str,
        depth: usize,
        chunks: &mut Vec<CodeChunk>
    ) {
        loop {
            let node = cursor.node();
            let node_type = node.kind();

            if matches!(node_type, "function_declaration" | "function" | "class_declaration" | "method_definition" | "arrow_function") {
                if let Ok(code) = self.extract_node_content(node, content) {
                    if code.len() > 30 {
                        let start_line = node.start_position().row;
                        let end_line = node.end_position().row;
                        let language = if file_name.ends_with(".ts") || file_name.ends_with(".tsx") {
                            Language::TypeScript
                        } else {
                            Language::JavaScript
                        };

                        let chunk = CodeChunk {
                            id: format!("{}_{}_{}", file_name, node_type, start_line),
                            content: code,
                            file_path: file_name.to_string(),
                            start_line: start_line as u32,
                            end_line: end_line as u32,
                            language,
                            embedding: None,
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert("file".to_string(), serde_json::json!(file_name));
                                meta.insert("node_type".to_string(), serde_json::json!(node_type));
                                meta.insert("depth".to_string(), serde_json::json!(depth));
                                meta
                            },
                        };
                        chunks.push(chunk);
                    }
                }
            }

            if depth < 2 && cursor.goto_first_child() {
                self.traverse_javascript_node(cursor, content, file_name, depth + 1, chunks);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    /// Extract content from a tree-sitter node
    fn extract_node_content(&self, node: tree_sitter::Node, content: &str) -> Result<String> {
        let start = node.start_byte();
        let end = node.end_byte();

        if start >= content.len() || end > content.len() || start >= end {
            return Err(Error::internal("Invalid node range".to_string()));
        }

        let code = content[start..end].trim();
        if code.is_empty() {
            return Err(Error::internal("Empty node content".to_string()));
        }

        Ok(code.to_string())
    }

    /// Fallback Rust chunking using regex patterns
    fn chunk_rust_fallback(&self, content: &str, file_name: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_block = Vec::new();
        let mut block_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            current_block.push(line.clone());

            // Detect significant Rust constructs
            if trimmed.starts_with("fn ") ||
               trimmed.starts_with("struct ") ||
               trimmed.starts_with("impl ") ||
               trimmed.starts_with("pub fn ") ||
               trimmed.starts_with("pub struct ") {

                if current_block.len() > 1 {
                    self.create_chunk(&current_block[..current_block.len()-1], block_start, i-1, file_name, Language::Rust, &mut chunks);
                    current_block = vec![line.clone()];
                    block_start = i;
                }
            }

            // End block detection (simple brace counting)
            let open_braces = line.chars().filter(|&c| c == '{').count();
            let close_braces = line.chars().filter(|&c| c == '}').count();

            if open_braces > 0 && close_braces == open_braces && current_block.len() > 3 {
                self.create_chunk(&current_block, block_start, i, file_name, Language::Rust, &mut chunks);
                current_block.clear();
                block_start = i + 1;
            }
        }

        if !current_block.is_empty() {
            self.create_chunk(&current_block, block_start, lines.len() - 1, file_name, Language::Rust, &mut chunks);
        }

        chunks
    }

    /// Fallback Python chunking
    fn chunk_python_fallback(&self, content: &str, file_name: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_function = Vec::new();
        let mut function_start = 0;
        let mut indent_level = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                if !current_function.is_empty() {
                    self.create_chunk(&current_function, function_start, i-1, file_name, Language::Python, &mut chunks);
                    current_function.clear();
                }
                current_function.push(line.clone());
                function_start = i;
                indent_level = line.chars().take_while(|c| c.is_whitespace()).count();
            } else if !current_function.is_empty() {
                let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();

                if current_indent <= indent_level && !line.chars().all(|c| c.is_whitespace()) {
                    self.create_chunk(&current_function, function_start, i-1, file_name, Language::Python, &mut chunks);
                    current_function.clear();
                } else {
                    current_function.push(line.clone());
                }
            }
        }

        if !current_function.is_empty() {
            self.create_chunk(&current_function, function_start, lines.len() - 1, file_name, Language::Python, &mut chunks);
        }

        chunks
    }

    /// Fallback JavaScript chunking
    fn chunk_javascript_fallback(&self, content: &str, file_name: &str, language: Language) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_function = Vec::new();
        let mut function_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("function ") ||
               trimmed.starts_with("const ") && trimmed.contains("=>") ||
               trimmed.starts_with("class ") {

                if !current_function.is_empty() {
                    self.create_chunk(&current_function, function_start, i-1, file_name, language.clone(), &mut chunks);
                    current_function.clear();
                }
                current_function.push(line.clone());
                function_start = i;
            } else if !current_function.is_empty() {
                current_function.push(line.clone());

                // Simple heuristic: end function on balanced braces
                let open_count = current_function.iter().map(|l| l.chars().filter(|&c| c == '{').count()).sum::<usize>();
                let close_count = current_function.iter().map(|l| l.chars().filter(|&c| c == '}').count()).sum::<usize>();

                if open_count > 0 && open_count == close_count && current_function.len() > 2 {
                    self.create_chunk(&current_function, function_start, i, file_name, language.clone(), &mut chunks);
                    current_function.clear();
                }
            }
        }

        if !current_function.is_empty() {
            self.create_chunk(&current_function, function_start, lines.len() - 1, file_name, language, &mut chunks);
        }

        chunks
    }

    /// Create a chunk from lines
    fn create_chunk(
        &self,
        lines: &[String],
        start_line: usize,
        end_line: usize,
        file_name: &str,
        language: Language,
        chunks: &mut Vec<CodeChunk>
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
            language,
            embedding: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("file".to_string(), serde_json::json!(file_name));
                meta.insert("chunk_type".to_string(), serde_json::json!("structural"));
                meta
            },
        };
        chunks.push(chunk);
    }
}