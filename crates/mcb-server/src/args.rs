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

/// Consolidated argument types.
pub mod consolidated;

pub use consolidated::{
    AgentAction, AgentArgs, IndexAction, IndexArgs, IssueEntityAction, IssueEntityArgs,
    IssueEntityResource, MemoryAction, MemoryArgs, MemoryResource, OrgEntityAction, OrgEntityArgs,
    OrgEntityResource, PlanEntityAction, PlanEntityArgs, PlanEntityResource, ProjectAction,
    ProjectArgs, ProjectResource, SearchArgs, SearchResource, SessionAction, SessionArgs,
    ValidateAction, ValidateArgs, ValidateScope, VcsAction, VcsArgs, VcsEntityAction,
    VcsEntityArgs, VcsEntityResource,
};
