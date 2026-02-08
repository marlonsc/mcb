//! Session Management
//!
//! Provides session isolation for MCB server connections.
//! Each client connection can have its own session context,
//! allowing for collection namespace prefixing and isolation.

mod manager;

pub use manager::{create_session_manager, SessionContext, SessionManager};
