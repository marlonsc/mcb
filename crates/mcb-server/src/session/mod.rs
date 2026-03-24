//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Session Management
//!
//! Provides session isolation for MCB server connections.
//! Each client connection can have its own session context,
//! allowing for collection namespace prefixing and isolation.

mod manager;

pub use manager::{SessionContext, SessionManager};
