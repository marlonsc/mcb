//! Domain layer constants

/// AST and tree-sitter node type constants.
pub mod ast;
/// Embedding dimension constants for each provider and model family.
pub mod embedding;
/// HTTP constants
pub mod http;
pub mod keys;
/// Language identifier constants
pub mod lang;
/// Search and BM25 algorithmic constants
pub mod search;
pub mod values;
pub use values::*;
