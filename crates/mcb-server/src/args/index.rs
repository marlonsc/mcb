//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

tool_enum! {
/// Actions available for the index tool.
pub enum IndexAction {
    /// Start a new indexing operation.
    Start,
    /// Start git-aware incremental indexing.
    GitIndex,
    /// Get the status of current indexing operation.
    Status,
    /// Clear the index for a collection.
    Clear,
}
}

tool_schema! {
/// Arguments for the index tool.
pub struct IndexArgs {
    /// Action to perform: start, `git_index`, status, clear.
    #[schemars(description = "Action to perform: start, git_index, status, clear")]
    pub action: IndexAction,

    /// Path to codebase directory (required for 'start' action).
    #[schemars(
        description = "Path to codebase directory (required for 'start' action)",
        with = "String"
    )]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub path: Option<String>,

    /// Collection name for the index.
    #[schemars(description = "Collection name for the index", with = "String")]
    #[validate(custom(
        function = "super::validate_collection_name",
        message = "Invalid collection name"
    ))]
    pub collection: Option<String>,

    /// File extensions to include (for 'start' action).
    #[schemars(
        description = "File extensions to include (for 'start' action)",
        with = "Vec<String>"
    )]
    pub extensions: Option<Vec<String>>,

    /// Directories to exclude (for 'start' action).
    #[schemars(
        description = "Directories to exclude (for 'start' action)",
        with = "Vec<String>"
    )]
    pub exclude_dirs: Option<Vec<String>>,

    /// Glob patterns for files/directories to exclude.
    #[schemars(
        description = "Glob patterns for files/directories to exclude",
        with = "Vec<String>"
    )]
    pub ignore_patterns: Option<Vec<String>>,

    /// Maximum file size to index (bytes).
    #[schemars(description = "Maximum file size to index (bytes)", with = "u64")]
    pub max_file_size: Option<u64>,

    /// Follow symbolic links during indexing.
    #[schemars(description = "Follow symbolic links during indexing", with = "bool")]
    pub follow_symlinks: Option<bool>,

    /// JWT token for authenticated requests.
    #[schemars(description = "JWT token for authenticated requests", with = "String")]
    pub token: Option<String>,
}
}
