//! Tool argument types for MCP server
//!
//! This module contains all the argument types used by the MCP tools.
//! These are extracted to improve code organization and maintainability.

use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

/// Arguments for the index_codebase tool
#[derive(Clone, Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for indexing a codebase directory")]
pub struct IndexCodebaseArgs {
    /// Path to the codebase directory to index
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(
        description = "Absolute or relative path to the directory containing code to index"
    )]
    pub path: String,
    /// Collection name for the indexed data
    #[schemars(description = "Name of the collection to store indexed data")]
    pub collection: Option<String>,
    /// File extensions to include (e.g., [\"rs\", \"py\", \"js\"])
    #[schemars(description = "Only index files with these extensions")]
    pub extensions: Option<Vec<String>>,
    /// Patterns to ignore during indexing
    #[schemars(description = "Glob patterns for files/directories to exclude")]
    pub ignore_patterns: Option<Vec<String>>,
    /// Maximum file size to index (in bytes)
    #[schemars(description = "Maximum size of files to index")]
    pub max_file_size: Option<u64>,
    /// Whether to follow symbolic links
    #[schemars(description = "Follow symbolic links during indexing")]
    pub follow_symlinks: Option<bool>,
    /// Optional JWT token for authentication
    #[schemars(description = "JWT token for authenticated requests")]
    pub token: Option<String>,
}

/// Search filters for narrowing down search results
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Filters to narrow down search results")]
pub struct SearchFiltersInput {
    /// Filter by file extensions (e.g., [\"rs\", \"py\", \"js\"])
    #[schemars(description = "Only include files with these extensions")]
    pub file_extensions: Option<Vec<String>>,
    /// Filter by programming languages
    #[schemars(description = "Only include files in these programming languages")]
    pub languages: Option<Vec<String>>,
    /// Exclude files matching these patterns
    #[schemars(description = "Exclude files matching these glob patterns")]
    pub exclude_patterns: Option<Vec<String>>,
    /// Minimum similarity score (0.0 to 1.0)
    #[validate(range(
        min = 0.0,
        max = 1.0,
        message = "Min score must be between 0.0 and 1.0"
    ))]
    #[schemars(description = "Minimum similarity score threshold")]
    pub min_score: Option<f32>,
}

/// Arguments for the search_code tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for searching code using natural language")]
pub struct SearchCodeArgs {
    /// Natural language query to search for
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Query must be between 1 and 1000 characters"
    ))]
    #[validate(custom(function = "validate_search_query", message = "Invalid search query"))]
    #[schemars(
        description = "The search query in natural language (e.g., 'find functions that handle authentication')"
    )]
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[validate(range(min = 1, max = 1000, message = "Limit must be between 1 and 1000"))]
    #[schemars(description = "Maximum number of search results to return")]
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Collection name to search in
    #[schemars(description = "Name of the collection to search")]
    pub collection: Option<String>,
    /// File extensions to search in
    #[schemars(description = "Only search in files with these extensions")]
    pub extensions: Option<Vec<String>>,
    /// Optional search filters
    #[schemars(description = "Optional filters to narrow down search results")]
    pub filters: Option<SearchFiltersInput>,
    /// Optional JWT token for authentication
    #[schemars(description = "JWT token for authenticated requests")]
    pub token: Option<String>,
}

/// Arguments for getting indexing status
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for checking indexing status")]
pub struct GetIndexingStatusArgs {
    /// Collection name (default: 'default')
    #[validate(length(
        min = 1,
        max = 100,
        message = "Collection name must be between 1 and 100 characters"
    ))]
    #[validate(custom(
        function = "validate_collection_name",
        message = "Invalid collection name"
    ))]
    #[schemars(description = "Name of the collection to check status for")]
    #[serde(default = "default_collection")]
    pub collection: String,
}

/// Arguments for clearing an index
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for clearing an index")]
pub struct ClearIndexArgs {
    /// Collection name to clear (default: 'default')
    #[validate(length(
        min = 1,
        max = 100,
        message = "Collection name must be between 1 and 100 characters"
    ))]
    #[validate(custom(
        function = "validate_collection_name",
        message = "Invalid collection name"
    ))]
    #[schemars(description = "Name of the collection to clear")]
    #[serde(default = "default_collection")]
    pub collection: String,
}

pub(crate) fn default_limit() -> usize {
    10
}

fn default_collection() -> String {
    "default".to_string()
}

// Custom validation functions

fn validate_file_path(path: &str) -> Result<(), validator::ValidationError> {
    if path.is_empty() {
        return Err(validator::ValidationError::new("Path cannot be empty"));
    }

    if path.contains("..") {
        return Err(validator::ValidationError::new(
            "Path cannot contain directory traversal",
        ));
    }

    // Check for sensitive system paths (but allow /home/ for user code)
    let sensitive_paths = ["/etc/", "/proc/", "/sys/", "/root/"];
    for sensitive in &sensitive_paths {
        if path.starts_with(sensitive) {
            return Err(validator::ValidationError::new(
                "Access to sensitive system paths is not allowed",
            ));
        }
    }

    Ok(())
}

fn validate_search_query(query: &str) -> Result<(), validator::ValidationError> {
    if query.is_empty() {
        return Err(validator::ValidationError::new(
            "Search query cannot be empty",
        ));
    }

    if query.len() > 1000 {
        return Err(validator::ValidationError::new(
            "Search query is too long (maximum 1000 characters)",
        ));
    }

    // Input validation to prevent injection attacks
    let dangerous_patterns = ["<script", "javascript:", "onload=", "onerror="];
    for pattern in &dangerous_patterns {
        if query.to_lowercase().contains(pattern) {
            return Err(validator::ValidationError::new(
                "Search query contains potentially dangerous content",
            ));
        }
    }

    Ok(())
}

fn validate_collection_name(name: &str) -> Result<(), validator::ValidationError> {
    if name.is_empty() {
        return Err(validator::ValidationError::new(
            "Collection name cannot be empty",
        ));
    }

    if name.len() > 100 {
        return Err(validator::ValidationError::new(
            "Collection name is too long (maximum 100 characters)",
        ));
    }

    // Only allow alphanumeric, underscore, and hyphen
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(validator::ValidationError::new(
            "Collection name can only contain letters, numbers, underscores, and hyphens",
        ));
    }

    Ok(())
}

/// Arguments for the validate_architecture tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for validating architecture rules")]
pub struct ValidateArchitectureArgs {
    /// Path to workspace root directory
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(description = "Absolute path to the workspace root directory")]
    pub path: String,

    /// Specific validators to run (optional, default: all)
    #[schemars(
        description = "List of validators to run: clean_architecture, solid, quality, organization, kiss, naming, documentation, performance, async_patterns"
    )]
    pub validators: Option<Vec<String>>,

    /// Minimum severity filter (optional, default: all)
    #[schemars(description = "Minimum severity level to report: error, warning, or info")]
    pub severity_filter: Option<String>,

    /// Exclude patterns (optional)
    #[schemars(description = "Glob patterns for files/directories to exclude from validation")]
    pub exclude_patterns: Option<Vec<String>>,
}

/// Arguments for the validate_file tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for validating a single file")]
pub struct ValidateFileArgs {
    /// Path to the file to validate
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(description = "Absolute path to the file to validate")]
    pub path: String,

    /// Specific validators to run (optional, default: all)
    #[schemars(
        description = "List of validators to run: clean_architecture, solid, quality, organization"
    )]
    pub validators: Option<Vec<String>>,
}

/// Arguments for the list_validators tool
///
/// This tool requires no parameters - it returns all available validators.
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "No parameters required - returns all available validators")]
pub struct ListValidatorsArgs {
    /// Reserved for future filtering capabilities
    #[schemars(description = "Reserved for future category filtering")]
    #[serde(default)]
    pub category: Option<String>,
}

/// Arguments for the get_validation_rules tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for getting validation rules")]
pub struct GetValidationRulesArgs {
    /// Category filter (optional)
    #[schemars(
        description = "Filter rules by category: clean_architecture, solid, quality, kiss, organization"
    )]
    pub category: Option<String>,
}

/// Arguments for the analyze_complexity tool
#[derive(Debug, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for analyzing code complexity")]
pub struct AnalyzeComplexityArgs {
    /// Path to the file to analyze
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(description = "Absolute path to the file to analyze")]
    pub path: String,

    /// Include function-level metrics (optional, default: false)
    #[schemars(description = "Whether to include per-function complexity metrics")]
    #[serde(default)]
    pub include_functions: bool,
}

/// Arguments for the `index_vcs_repository` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for indexing a VCS repository")]
pub struct IndexVcsRepositoryArgs {
    /// Path to VCS repository
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(description = "Absolute path to the VCS repository")]
    pub path: String,

    /// Branches to index (default: default branch only)
    #[serde(default)]
    #[schemars(description = "List of branches to index (empty = default branch only)")]
    pub branches: Vec<String>,

    /// Also index commit messages
    #[serde(default)]
    #[schemars(description = "Whether to index commit messages for search")]
    pub include_commits: bool,
}

/// Arguments for the `search_branch` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Parameters for searching code within a specific branch")]
pub struct SearchBranchArgs {
    /// Repository ID from index_vcs_repository
    #[validate(length(min = 1, message = "repository_id cannot be empty"))]
    #[schemars(description = "Repository ID returned by index_vcs_repository")]
    pub repository_id: String,

    /// Branch name to search
    #[validate(length(min = 1, message = "branch cannot be empty"))]
    #[schemars(description = "Name of the branch to search within")]
    pub branch: String,

    /// Search query
    #[validate(length(min = 1, message = "query cannot be empty"))]
    #[schemars(description = "The search query")]
    pub query: String,

    /// Maximum number of results
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schemars(description = "Maximum number of results to return (default: 10)")]
    pub limit: usize,
}

/// Arguments for the `list_repositories` tool
#[derive(Debug, Clone, Default, Deserialize, JsonSchema, Validate)]
#[schemars(description = "No parameters required - returns all indexed repositories")]
pub struct ListRepositoriesArgs {}

/// Arguments for the `compare_branches` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Compare two branches and return the diff")]
pub struct CompareBranchesArgs {
    #[validate(length(min = 1, message = "path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(description = "Path to the VCS repository")]
    pub path: String,

    #[validate(length(min = 1, message = "base_branch cannot be empty"))]
    #[schemars(description = "Base branch for comparison")]
    pub base_branch: String,

    #[validate(length(min = 1, message = "head_branch cannot be empty"))]
    #[schemars(description = "Head branch for comparison")]
    pub head_branch: String,
}

/// Arguments for the `analyze_impact` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Analyze the impact of changes between two refs")]
pub struct AnalyzeImpactArgs {
    #[validate(length(min = 1, message = "path cannot be empty"))]
    #[validate(custom(function = "validate_file_path", message = "Invalid file path"))]
    #[schemars(description = "Path to the VCS repository")]
    pub path: String,

    #[validate(length(min = 1, message = "base_ref cannot be empty"))]
    #[schemars(description = "Base ref (branch, tag, or commit)")]
    pub base_ref: String,

    #[validate(length(min = 1, message = "head_ref cannot be empty"))]
    #[schemars(description = "Head ref (branch, tag, or commit)")]
    pub head_ref: String,
}

/// Arguments for the `store_observation` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Store an observation in semantic memory")]
pub struct StoreObservationArgs {
    #[validate(length(min = 1, max = 10000, message = "Content must be 1-10000 chars"))]
    #[schemars(description = "The observation content to store")]
    pub content: String,

    #[schemars(description = "Type of observation: code, decision, context, error, summary")]
    pub observation_type: String,

    #[serde(default)]
    #[schemars(description = "Tags for categorizing the observation")]
    pub tags: Vec<String>,

    #[schemars(description = "Session ID to associate with this observation")]
    pub session_id: Option<String>,

    #[schemars(description = "Repository ID for context")]
    pub repo_id: Option<String>,

    #[schemars(description = "File path related to this observation")]
    pub file_path: Option<String>,

    #[schemars(description = "VCS branch related to this observation")]
    pub branch: Option<String>,

    #[schemars(description = "VCS commit related to this observation")]
    pub commit: Option<String>,
}

/// Arguments for the `search_memories` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Search observations using semantic similarity")]
pub struct SearchMemoriesArgs {
    #[validate(length(min = 1, max = 1000, message = "Query must be 1-1000 chars"))]
    #[schemars(description = "Search query for semantic matching")]
    pub query: String,

    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schemars(description = "Maximum number of results (default: 10)")]
    pub limit: usize,

    #[schemars(description = "Filter by tags")]
    pub tags: Option<Vec<String>>,

    #[schemars(description = "Filter by observation type")]
    pub observation_type: Option<String>,

    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,
}

/// Arguments for the `get_session_summary` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Get a session summary by session ID")]
pub struct GetSessionSummaryArgs {
    #[validate(length(min = 1, message = "session_id cannot be empty"))]
    #[schemars(description = "The session ID to get summary for")]
    pub session_id: String,
}

/// Arguments for the `create_session_summary` tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Create a summary of a coding session")]
pub struct CreateSessionSummaryArgs {
    #[validate(length(min = 1, message = "session_id cannot be empty"))]
    #[schemars(description = "Session ID to create summary for")]
    pub session_id: String,

    #[serde(default)]
    #[schemars(description = "Key topics discussed in the session")]
    pub topics: Vec<String>,

    #[serde(default)]
    #[schemars(description = "Decisions made during the session")]
    pub decisions: Vec<String>,

    #[serde(default)]
    #[schemars(description = "Next steps or action items")]
    pub next_steps: Vec<String>,

    #[serde(default)]
    #[schemars(description = "Key files worked on during the session")]
    pub key_files: Vec<String>,
}

pub mod memory;
pub use memory::*;

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Create an agent session record")]
pub struct CreateAgentSessionArgs {
    #[validate(length(min = 1, message = "session_summary_id cannot be empty"))]
    #[schemars(description = "The parent session summary ID")]
    pub session_summary_id: String,

    #[validate(length(min = 1, message = "agent_type cannot be empty"))]
    #[schemars(description = "Type of agent: sisyphus, oracle, or explore")]
    pub agent_type: String,

    #[validate(length(min = 1, message = "model cannot be empty"))]
    #[schemars(description = "Model name used for this agent session")]
    pub model: String,

    #[schemars(description = "Parent agent session ID for delegation chains")]
    pub parent_session_id: Option<String>,

    #[schemars(description = "Summary of the prompt given to the agent")]
    pub prompt_summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Get an agent session by ID")]
pub struct GetAgentSessionArgs {
    #[validate(length(min = 1, message = "id cannot be empty"))]
    #[schemars(description = "The agent session ID")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Update an agent session (typically to mark completion)")]
pub struct UpdateAgentSessionArgs {
    #[validate(length(min = 1, message = "id cannot be empty"))]
    #[schemars(description = "The agent session ID to update")]
    pub id: String,

    #[schemars(description = "New status: active, completed, or failed")]
    pub status: Option<String>,

    #[schemars(description = "Summary of the agent's result")]
    pub result_summary: Option<String>,

    #[schemars(description = "Token count used in this session")]
    pub token_count: Option<i64>,

    #[schemars(description = "Number of tool calls made")]
    pub tool_calls_count: Option<i64>,

    #[schemars(description = "Number of delegations made")]
    pub delegations_count: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "List agent sessions with optional filters")]
pub struct ListAgentSessionsArgs {
    #[schemars(description = "Filter by parent session summary ID")]
    pub session_summary_id: Option<String>,

    #[schemars(description = "Filter by parent agent session ID")]
    pub parent_session_id: Option<String>,

    #[schemars(description = "Filter by agent type: sisyphus, oracle, or explore")]
    pub agent_type: Option<String>,

    #[schemars(description = "Filter by status: active, completed, or failed")]
    pub status: Option<String>,

    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schemars(description = "Maximum number of results (default: 10)")]
    pub limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Store a tool call record")]
pub struct StoreToolCallArgs {
    #[validate(length(min = 1, message = "session_id cannot be empty"))]
    #[schemars(description = "The agent session ID this tool call belongs to")]
    pub session_id: String,

    #[validate(length(min = 1, message = "tool_name cannot be empty"))]
    #[schemars(description = "Name of the tool that was called")]
    pub tool_name: String,

    #[schemars(description = "Summary of the parameters passed")]
    pub params_summary: Option<String>,

    #[schemars(description = "Whether the tool call succeeded")]
    pub success: bool,

    #[schemars(description = "Error message if the tool call failed")]
    pub error_message: Option<String>,

    #[schemars(description = "Duration of the tool call in milliseconds")]
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Store a delegation record")]
pub struct StoreDelegationArgs {
    #[validate(length(min = 1, message = "parent_session_id cannot be empty"))]
    #[schemars(description = "The parent agent session ID")]
    pub parent_session_id: String,

    #[validate(length(min = 1, message = "child_session_id cannot be empty"))]
    #[schemars(description = "The child agent session ID")]
    pub child_session_id: String,

    #[validate(length(min = 1, message = "prompt cannot be empty"))]
    #[schemars(description = "The delegation prompt")]
    pub prompt: String,

    #[schemars(description = "Result from the delegated agent")]
    pub result: Option<String>,

    #[schemars(description = "Whether the delegation succeeded")]
    pub success: bool,

    #[schemars(description = "Duration of the delegation in milliseconds")]
    pub duration_ms: Option<i64>,
}
