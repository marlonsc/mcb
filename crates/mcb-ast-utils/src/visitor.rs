//! Node Visitor Pattern
//!
//! Provides a visitor pattern for traversing AST nodes.

use tree_sitter::Node;

/// Visitor trait for AST node traversal.
///
/// Implement this trait to process specific node types during tree traversal.
///
/// # Example
///
/// ```ignore
/// impl NodeVisitor for MyVisitor {
///     type Context = ();
///     fn visit(&mut self, node: Node<'_>, _source: &[u8], _ctx: &mut Self::Context) -> bool { true }
/// }
/// ```
pub trait NodeVisitor {
    /// Context type passed through the traversal
    type Context;

    /// Visit a node and optionally return a result
    ///
    /// # Arguments
    /// * `node` - The current AST node
    /// * `source` - The source code bytes
    /// * `ctx` - Mutable context passed through traversal
    ///
    /// # Returns
    /// `true` to continue visiting children, `false` to skip children
    fn visit(&mut self, node: Node<'_>, source: &[u8], ctx: &mut Self::Context) -> bool;

    /// Called after all children have been visited
    ///
    /// # Arguments
    /// * `node` - The current AST node
    /// * `source` - The source code bytes
    /// * `ctx` - Mutable context passed through traversal
    fn leave(&mut self, _node: Node<'_>, _source: &[u8], _ctx: &mut Self::Context) {}
}

/// A simple visitor that collects nodes of a specific kind
pub struct KindCollector {
    /// The node kind to collect
    pub target_kind: String,
    /// Collected node ranges (`start_byte`, `end_byte`, `start_row`, `start_col`)
    pub matches: Vec<NodeMatch>,
}

/// Information about a matched node
#[derive(Debug, Clone)]
pub struct NodeMatch {
    /// Start byte offset
    pub start_byte: usize,
    /// End byte offset
    pub end_byte: usize,
    /// Start line (0-indexed)
    pub start_line: usize,
    /// Start column (0-indexed)
    pub start_column: usize,
    /// End line (0-indexed)
    pub end_line: usize,
    /// End column (0-indexed)
    pub end_column: usize,
    /// The node's text content
    pub text: String,
}

impl KindCollector {
    /// Create a new kind collector
    pub fn new(target_kind: &str) -> Self {
        Self {
            target_kind: target_kind.to_string(),
            matches: Vec::new(),
        }
    }
}

impl NodeVisitor for KindCollector {
    type Context = ();

    fn visit(&mut self, node: Node<'_>, source: &[u8], _ctx: &mut Self::Context) -> bool {
        if node.kind() == self.target_kind {
            let text = node.utf8_text(source).map(String::from).unwrap_or_default();

            self.matches.push(NodeMatch {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_line: node.start_position().row,
                start_column: node.start_position().column,
                end_line: node.end_position().row,
                end_column: node.end_position().column,
                text,
            });
        }
        true // Always continue to children
    }
}

/// A visitor that counts nodes of each kind
pub struct KindCounter {
    /// Map of kind to count
    pub counts: std::collections::HashMap<String, usize>,
}

impl Default for KindCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl KindCounter {
    /// Create a new kind counter
    pub fn new() -> Self {
        Self {
            counts: std::collections::HashMap::new(),
        }
    }

    /// Get the count for a specific kind
    pub fn count(&self, kind: &str) -> usize {
        self.counts.get(kind).copied().unwrap_or(0)
    }
}

impl NodeVisitor for KindCounter {
    type Context = ();

    fn visit(&mut self, node: Node<'_>, _source: &[u8], _ctx: &mut Self::Context) -> bool {
        *self.counts.entry(node.kind().to_string()).or_insert(0) += 1;
        true
    }
}

// Tests moved to tests/unit/visitor_tests.rs
