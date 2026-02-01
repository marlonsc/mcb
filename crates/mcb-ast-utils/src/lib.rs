//! MCB AST Utilities
//!
//! Provides AST traversal and analysis utilities for MCP Context Browser.
//! Built on tree-sitter for direct AST access and analysis.
//!
//! ## Features
//!
//! - **Tree Walking**: Depth-first traversal with visitor pattern
//! - **Cursor Utilities**: Navigation helpers for tree-sitter cursors
//! - **Symbol Extraction**: Extract function, class, and other symbols
//! - **Complexity Analysis**: Calculate cyclomatic and cognitive complexity
//!
//! ## Example
//!
//! ```no_run
//! use mcb_ast_utils::{TreeWalker, SymbolExtractor, ComplexityAnalyzer};
//! use mcb_language_support::LanguageId;
//! use tree_sitter::Parser;
//!
//! fn example() {
//!     let mut parser = Parser::new();
//!     parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
//!
//!     let code = "fn main() { println!(\"Hello\"); }";
//!     let tree = parser.parse(code, None).unwrap();
//!
//!     // Extract symbols
//!     let symbols = SymbolExtractor::extract(&tree, code.as_bytes(), LanguageId::Rust);
//!     println!("Found {} symbols", symbols.len());
//!
//!     // Analyze complexity
//!     let metrics = ComplexityAnalyzer::analyze(tree.root_node(), LanguageId::Rust);
//!     println!("Cyclomatic complexity: {}", metrics.cyclomatic);
//! }
//! ```

pub mod complexity;
pub mod cursor;
pub mod error;
pub mod symbols;
pub mod visitor;
pub mod walker;

// Re-export main types
pub use complexity::{ComplexityAnalyzer, ComplexityMetrics, count_parameters, nesting_depth_at};
pub use cursor::CursorUtils;
pub use error::{AstError, Result};
pub use symbols::{SymbolExtractor, SymbolInfo, SymbolKind};
pub use visitor::{KindCollector, KindCounter, NodeMatch, NodeVisitor};
pub use walker::TreeWalker;
