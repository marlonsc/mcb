//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Domain layer constants

/// AST and tree-sitter node type constants.
pub mod ast;
/// Embedding dimension constants for each provider and model family.
pub mod embedding;
/// HTTP constants
pub mod http;
/// I/O and buffer size constants.
pub mod io;
pub mod keys;
/// Language identifier constants
pub mod lang;
/// Search and BM25 algorithmic constants
pub mod search;
/// Time validation and boundary constants.
pub mod time;
pub mod values;
pub use values::*;
