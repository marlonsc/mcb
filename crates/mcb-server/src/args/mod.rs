//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Argument types and validators for MCP tool requests.
//!
//! This module provides request argument types for all MCP tools and validation
//! functions to ensure input safety and correctness.

pub(crate) mod validation;
pub(crate) use validation::{validate_collection_name, validate_file_path};

/// Schema helper types for argument definitions.
pub mod schema_helpers;

/// Consolidated argument types (legacy, re-exports from split modules).
pub mod consolidated;

/// Agent activity logging argument types.
pub mod agent;
/// Unified entity CRUD argument types.
pub mod entity;
/// Index operations argument types.
pub mod index;
/// Memory storage and retrieval argument types.
pub mod memory;
/// Project workflow argument types.
pub mod project;
/// Search operations argument types.
pub mod search;
/// Session lifecycle argument types.
pub mod session;
/// Validation and analysis argument types.
pub mod validate;
/// Version control operations argument types.
pub mod vcs;

// Consolidate arguments from all modules (User refactor)
pub use consolidated::{
    AgentAction, AgentArgs, EntityAction, EntityArgs, EntityResource, IndexAction, IndexArgs,
    IssueEntityAction, IssueEntityArgs, IssueEntityResource, MemoryAction, MemoryArgs,
    MemoryResource, OrgEntityAction, OrgEntityArgs, OrgEntityResource, PlanEntityAction,
    PlanEntityArgs, PlanEntityResource, ProjectAction, ProjectArgs, ProjectResource, SearchArgs,
    SearchResource, SessionAction, SessionArgs, ValidateAction, ValidateArgs, ValidateScope,
    VcsAction, VcsArgs, VcsEntityAction, VcsEntityArgs, VcsEntityResource,
};
