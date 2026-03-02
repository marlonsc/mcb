//! Workspace-wide constants.
//!
//! Domain layer constants centralized in mcb-utils (Layer 0).

/// AST and tree-sitter node type constants.
pub mod ast;
/// Authentication constants (API keys, token prefixes).
pub mod auth;
/// Cryptographic constants (AES-GCM, key sizes).
pub mod crypto;
/// Embedding dimension constants for each provider and model family.
pub mod embedding;
/// Event bus and messaging constants.
pub mod events;
/// HTTP constants.
pub mod http;
/// I/O and buffer size constants.
pub mod io;
/// Key name constants.
pub mod keys;
/// Language identifier constants.
pub mod lang;
/// Resource limits constants.
pub mod limits;
/// Search and BM25 algorithmic constants.
pub mod search;
/// Time validation and boundary constants.
pub mod time;
/// Use case / business logic constants.
pub mod use_cases;
/// Commonly used constant values.
pub mod values;
/// Re-export all values for convenience.
pub use values::*;
