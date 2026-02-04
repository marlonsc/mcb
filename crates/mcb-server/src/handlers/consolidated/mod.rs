//! Consolidated MCP tool handlers.
//!
//! This module contains unified handlers for MCP tool operations,
//! organized by domain: agent, index, memory, project, search, session, validate, vcs.

pub mod agent;
pub mod index;
pub mod memory;
pub mod project;
pub mod search;
pub mod session;
pub mod validate;
pub mod vcs;

pub use agent::AgentHandler;
pub use index::IndexHandler;
pub use memory::MemoryHandler;
pub use project::ProjectHandler;
pub use search::SearchHandler;
pub use session::SessionHandler;
pub use validate::ValidateHandler;
pub use vcs::VcsHandler;
