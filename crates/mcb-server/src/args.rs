//! Argument types and validators for MCP tool requests.
//!
//! This module provides request argument types for all MCP tools and validation
//! functions to ensure input safety and correctness.

use validator::ValidationError;

pub(crate) fn validate_file_path(path: &str) -> Result<(), ValidationError> {
    if path.contains("..") || path.contains('\0') {
        let mut err = ValidationError::new("invalid_path");
        err.message = Some("Path traversal is not allowed".into());
        return Err(err);
    }
    Ok(())
}

pub(crate) fn validate_collection_name(collection: &str) -> Result<(), ValidationError> {
    if collection.is_empty() {
        let mut err = ValidationError::new("invalid_collection");
        err.message = Some("Collection name cannot be empty".into());
        return Err(err);
    }
    let valid = collection
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
    if !valid {
        let mut err = ValidationError::new("invalid_collection");
        err.message = Some("Collection name contains invalid characters".into());
        return Err(err);
    }
    Ok(())
}

pub(crate) mod macros;

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
