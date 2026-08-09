//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Session handler for agent session management.
//!
//! This module provides a unified handler for agent session MCP tool operations.

mod common;
mod create;
mod get;
mod handler;
mod list;
mod summarize;
mod update;

pub use handler::SessionHandler;
