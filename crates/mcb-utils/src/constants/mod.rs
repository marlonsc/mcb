//! Workspace-wide constants.
//!
//! Domain layer constants centralized in mcb-utils (Layer 0).

/// AST and tree-sitter node type constants.
pub mod ast;
/// Authentication constants (API keys, token prefixes).
pub mod auth;
/// Cryptographic constants (AES-GCM, key sizes).
pub mod crypto;
/// Display formatting and presentation constants.
pub mod display;
/// Embedding dimension constants for each provider and model family.
pub mod embedding;
/// Event bus and messaging constants.
pub mod events;
/// Custom HTTP header name constants for execution context / provenance.
pub mod headers;
/// HTTP constants.
pub mod http;
/// IDE / Agent program identifier constants.
pub mod ide;
/// I/O and buffer size constants.
pub mod io;
/// Key name constants.
pub mod keys;
/// Language identifier constants.
pub mod lang;
/// Resource limits constants.
pub mod limits;
/// MCP and JSON-RPC protocol constants.
pub mod protocol;
/// Search and BM25 algorithmic constants.
pub mod search;
/// Test constants, fixture values, and timeout defaults.
pub mod testing;
/// Time validation and boundary constants.
pub mod time;
/// Use case / business logic constants.
pub mod use_cases;
/// Validation constants centralized from mcb-validate.
pub mod validate;
/// Commonly used constant values.
pub mod values;
/// VCS and git-related constants.
pub mod vcs;
/// Vector store configuration constants.
pub mod vector_store;
/// Re-export all values for convenience.
pub use values::*;

// `define_str_consts!` macro is defined in `crate::macros` and available via `#[macro_use]`.
