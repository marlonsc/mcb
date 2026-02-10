//! MCP tool handlers.
//!
//! This module contains unified handlers for MCP tool operations,
//! organized by domain: agent, index, memory, project, search, session, validate, vcs.

pub mod agent;
pub mod index;
pub mod memory;
/// Plan entity CRUD handler.
pub mod plan_entity;
pub mod project;
pub mod search;
pub mod session;
pub mod validate;
pub mod vcs;
/// VCS entity CRUD handler.
pub mod vcs_entity;

pub use agent::AgentHandler;
pub use index::IndexHandler;
pub use memory::MemoryHandler;
pub use plan_entity::PlanEntityHandler;
pub use project::ProjectHandler;
pub use search::SearchHandler;
pub use session::SessionHandler;
pub use validate::ValidateHandler;
pub use vcs::VcsHandler;
pub use vcs_entity::VcsEntityHandler;
