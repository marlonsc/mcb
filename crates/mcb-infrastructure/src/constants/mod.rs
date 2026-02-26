//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Infrastructure layer constants. Domain-specific constants are in `mcb_domain::constants`.

/// Abstract syntax tree (AST) related constants.
pub mod ast;

/// Authentication and authorization constants.
pub mod auth;

/// Cryptographic operation constants.
pub mod crypto;

/// Event system and messaging constants.
pub mod events;

/// Syntax highlighting constants.
pub mod highlight;

/// HTTP server and client constants.
pub mod http;

/// Programming language and syntax constants.
pub mod lang;

/// System limits and constraints.
pub mod limits;

/// Search and indexing constants.
pub mod search;

/// Use case service constants (indexing, memory, search).
pub mod use_cases;
