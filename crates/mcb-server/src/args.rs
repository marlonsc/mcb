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

#[allow(missing_docs)]
pub mod agent;

#[allow(missing_docs)]
pub mod consolidated;

#[allow(missing_docs)]
pub mod entity;

#[allow(missing_docs)]
pub mod index;

#[allow(missing_docs)]
pub mod memory;

#[allow(missing_docs)]
pub mod project;

#[allow(missing_docs)]
pub mod search;

#[allow(missing_docs)]
pub mod session;

#[allow(missing_docs)]
pub mod validate;

#[allow(missing_docs)]
pub mod vcs;

pub use agent::{AgentAction, AgentArgs};
pub use entity::{
    EntityAction, EntityArgs, EntityResource, IssueEntityAction, IssueEntityArgs,
    IssueEntityResource, OrgEntityAction, OrgEntityArgs, OrgEntityResource, PlanEntityAction,
    PlanEntityArgs, PlanEntityResource, VcsEntityAction, VcsEntityArgs, VcsEntityResource,
};
pub use index::{IndexAction, IndexArgs};
pub use memory::{MemoryAction, MemoryArgs, MemoryResource};
pub use project::{ProjectAction, ProjectArgs, ProjectResource};
pub use search::{SearchArgs, SearchResource};
pub use session::{SessionAction, SessionArgs};
pub use validate::{ValidateAction, ValidateArgs, ValidateScope};
pub use vcs::{VcsAction, VcsArgs};
