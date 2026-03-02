//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Infrastructure layer constants. Domain-specific constants are in `mcb_domain::constants`.

/// Authentication and authorization constants.
pub mod auth;

/// Cryptographic operation constants.
pub mod crypto;

/// Event system and messaging constants.
pub mod events;

/// HTTP server and client constants.
pub mod http;

/// System limits and constraints.
pub mod limits;

/// Use case service constants (indexing, memory, search).
pub mod use_cases;
