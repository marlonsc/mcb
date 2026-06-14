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

    /// Repository identifier for collection auto-resolution (injected by context, hidden from MCP schema).
    #[schemars(skip)]
    pub repo_id: Option<String>,

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

// ---------------------------------------------------------------------------
// MCP-facing single-purpose tools
// ---------------------------------------------------------------------------

tool_action! {
    /// Arguments for the `index_repo` tool.
    pub struct IndexRepoArgs => IndexArgs {
        #[schemars(description = "File extensions to include", with = "Vec<String>")]
        extensions: Option<Vec<String>>,
        #[schemars(description = "Directories to exclude", with = "Vec<String>")]
        exclude_dirs: Option<Vec<String>>,
        #[schemars(description = "Glob patterns to ignore", with = "Vec<String>")]
        ignore_patterns: Option<Vec<String>>,
        #[schemars(description = "Maximum file size in bytes", with = "u64")]
        max_file_size: Option<u64>,
        #[schemars(description = "Follow symbolic links", with = "bool")]
        follow_symlinks: Option<bool>
        ;
        hidden { path: Option<String>, collection: Option<String>, repo_id: Option<String>, token: Option<String> }
        ;
        convert |a| {
            action: IndexAction::Start, extensions: a.extensions,
            exclude_dirs: a.exclude_dirs, ignore_patterns: a.ignore_patterns,
            max_file_size: a.max_file_size, follow_symlinks: a.follow_symlinks,
        }
    }
}

tool_action! {
    /// Arguments for the `index_status` tool.
    pub struct IndexStatusArgs => IndexArgs {
        ;
        hidden { path: Option<String>, collection: Option<String>, repo_id: Option<String>, token: Option<String> }
        ;
        convert |a| {
            action: IndexAction::Status, extensions: None, exclude_dirs: None,
            ignore_patterns: None, max_file_size: None, follow_symlinks: None,
        }
    }
}

tool_action! {
    /// Arguments for the `clear_index` tool.
    pub struct ClearIndexArgs => IndexArgs {
        ;
        hidden { path: Option<String>, collection: Option<String>, repo_id: Option<String>, token: Option<String> }
        ;
        convert |a| {
            action: IndexAction::Clear, extensions: None, exclude_dirs: None,
            ignore_patterns: None, max_file_size: None, follow_symlinks: None,
        }
    }
}
