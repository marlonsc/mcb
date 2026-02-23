//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! AST Core Types Module
//!
//! Core data structures for representing AST nodes and parsing results.

use std::collections::HashMap;

/// Unified AST node representation across all languages
#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    /// Node type (function, class, variable, etc.)
    pub kind: String,
    /// Node name (function name, variable name, etc.)
    pub name: Option<String>,
    /// Source code span (start/end positions)
    pub span: Span,
    /// Child nodes
    pub children: Vec<AstNode>,
    /// Additional metadata (language-specific)
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Source code position span
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// Start position of the span
    pub start: Position,
    /// End position of the span
    pub end: Position,
}

/// Position in source code
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Absolute byte offset in the source file
    pub byte_offset: usize,
}

/// AST parsing result
#[derive(Debug)]
pub struct AstParseResult {
    /// Root node of the parsed AST
    pub root: AstNode,
    /// List of parsing errors encountered
    pub errors: Vec<String>,
}
