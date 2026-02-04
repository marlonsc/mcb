//!
//! Argument types for MCP tools.
//! Consolidated tool arguments live in the `consolidated` module.

use validator::ValidationError;

pub mod consolidated;
pub use consolidated::*;

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
