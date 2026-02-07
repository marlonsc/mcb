use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

// =============================================================================
// Index Tool - Consolidates: index_codebase, get_indexing_status, clear_index
// =============================================================================

/// Actions available for the index tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexAction {
    /// Start a new indexing operation.
    Start,
    /// Get the status of current indexing operation.
    Status,
    /// Clear the index for a collection.
    Clear,
}

/// Arguments for the index tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct IndexArgs {
    /// Action to perform: start (index), status (check progress), clear (remove index).
    #[schemars(
        description = "Action to perform: start (index), status (check progress), clear (remove index)"
    )]
    pub action: IndexAction,

    /// Path to codebase directory (required for 'start' action).
    #[schemars(description = "Path to codebase directory (required for 'start' action)")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub path: Option<String>,

    /// Collection name for the index.
    #[schemars(description = "Collection name for the index")]
    #[validate(custom(
        function = "super::validate_collection_name",
        message = "Invalid collection name"
    ))]
    pub collection: Option<String>,

    /// File extensions to include (for 'start' action).
    #[schemars(description = "File extensions to include (for 'start' action)")]
    pub extensions: Option<Vec<String>>,

    /// Directories to exclude (for 'start' action).
    #[schemars(description = "Directories to exclude (for 'start' action)")]
    pub exclude_dirs: Option<Vec<String>>,

    /// Glob patterns for files/directories to exclude.
    #[schemars(description = "Glob patterns for files/directories to exclude")]
    pub ignore_patterns: Option<Vec<String>>,

    /// Maximum file size to index (bytes).
    #[schemars(description = "Maximum file size to index (bytes)")]
    pub max_file_size: Option<u64>,

    /// Follow symbolic links during indexing.
    #[schemars(description = "Follow symbolic links during indexing")]
    pub follow_symlinks: Option<bool>,

    /// JWT token for authenticated requests.
    #[schemars(description = "JWT token for authenticated requests")]
    pub token: Option<String>,
}

// =============================================================================
// Search Tool - Consolidates: search_code, search_memories, memory_search
// =============================================================================

/// Resources available for semantic search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchResource {
    /// Search across the indexed codebase.
    Code,
    /// Search across the memory (observations).
    Memory,
}

/// Arguments for the search tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct SearchArgs {
    /// Natural language search query.
    #[schemars(description = "Natural language search query")]
    #[validate(length(min = 1))]
    pub query: String,

    /// Resource to search: code or memory.
    #[schemars(description = "Resource to search: code or memory")]
    pub resource: SearchResource,

    /// Collection name.
    #[schemars(description = "Collection name")]
    pub collection: Option<String>,

    /// File extensions to include (code search only).
    #[schemars(description = "File extensions to include (code search only)")]
    pub extensions: Option<Vec<String>>,

    /// Additional search filters.
    #[schemars(description = "Additional search filters")]
    pub filters: Option<Vec<String>>,

    /// Maximum results to return.
    #[schemars(description = "Maximum results to return")]
    pub limit: Option<u32>,

    /// Minimum similarity score (0.0-1.0).
    #[schemars(description = "Minimum similarity score (0.0-1.0)")]
    #[validate(range(min = 0.0, max = 1.0, message = "Min score must be 0.0-1.0"))]
    pub min_score: Option<f32>,

    /// Filter by tags (for memory search).
    #[schemars(description = "Filter by tags (for memory search)")]
    pub tags: Option<Vec<String>>,

    /// Filter by session ID (for memory search).
    #[schemars(description = "Filter by session ID (for memory search)")]
    pub session_id: Option<String>,

    /// JWT token for authenticated requests.
    #[schemars(description = "JWT token for authenticated requests")]
    pub token: Option<String>,
}

// =============================================================================
// Validate Tool - Consolidates: validate_*, list_validators, analyze_complexity
// =============================================================================

/// Actions available for the validate tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidateAction {
    /// Run architectural validation rules.
    Run,
    /// List available validation rules.
    ListRules,
    /// Analyze code complexity (cyclomatic, cognitive).
    Analyze,
}

/// Scope for the validate action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidateScope {
    /// Validate a single file.
    File,
    /// Validate an entire project.
    Project,
}

/// Arguments for the validate tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct ValidateArgs {
    /// Action: run (validate), list_rules, analyze (complexity).
    #[schemars(description = "Action: run (validate), list_rules, analyze (complexity)")]
    pub action: ValidateAction,

    /// Scope: file or project.
    #[schemars(description = "Scope: file or project")]
    pub scope: Option<ValidateScope>,

    /// Path to file or project directory.
    #[schemars(description = "Path to file or project directory")]
    pub path: Option<String>,

    /// Specific rules to run (empty = all).
    #[schemars(description = "Specific rules to run (empty = all)")]
    pub rules: Option<Vec<String>>,

    /// Rule category filter.
    #[schemars(description = "Rule category filter")]
    pub category: Option<String>,
}

// =============================================================================
// Memory Tool - Consolidates all memory_* tools (14 tools → 1)
// =============================================================================

/// Actions available for the memory tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAction {
    /// Store a new memory item.
    Store,
    /// Get a specific memory item by ID.
    Get,
    /// List memory items with filters.
    List,
    /// Get a timeline of memory items.
    Timeline,
    /// Inject relevant memory items into context.
    Inject,
}

/// Resource types for the memory tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryResource {
    /// General observation.
    Observation,
    /// Tool execution result.
    Execution,
    /// Architectural quality gate.
    QualityGate,
    /// Common error pattern.
    ErrorPattern,
    /// Session metadata.
    Session,
}

/// Arguments for the memory tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct MemoryArgs {
    /// Action: store, get, list, timeline, inject.
    #[schemars(description = "Action: store, get, list, timeline, inject")]
    pub action: MemoryAction,

    /// Resource type: observation, execution, quality_gate, error_pattern, session.
    #[schemars(
        description = "Resource type: observation, execution, quality_gate, error_pattern, session"
    )]
    pub resource: MemoryResource,

    /// Data payload for store actions (JSON object).
    #[schemars(description = "Data payload for store actions (JSON object)")]
    pub data: Option<serde_json::Value>,

    /// Resource IDs for get action.
    #[schemars(description = "Resource IDs for get action")]
    pub ids: Option<Vec<String>>,

    /// Filter by project ID.
    #[schemars(description = "Filter by project ID")]
    pub project_id: Option<String>,

    /// Filter by repository ID.
    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,

    /// Filter by session ID.
    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    /// Filter by tags.
    #[schemars(description = "Filter by tags")]
    pub tags: Option<Vec<String>>,

    /// Query string for list/search actions.
    #[schemars(description = "Query string for list/search actions")]
    pub query: Option<String>,

    /// Anchor observation ID (for timeline action).
    #[schemars(description = "Anchor observation ID (for timeline action)")]
    pub anchor_id: Option<String>,

    /// Timeline depth before the anchor (default: 5).
    #[schemars(description = "Timeline depth before the anchor (default: 5)")]
    pub depth_before: Option<usize>,

    /// Timeline depth after the anchor (default: 5).
    #[schemars(description = "Timeline depth after the anchor (default: 5)")]
    pub depth_after: Option<usize>,

    /// Time window in seconds (for timeline action).
    #[schemars(description = "Time window in seconds (for timeline action)")]
    pub window_secs: Option<i64>,

    /// Observation types to include (inject action).
    #[schemars(description = "Observation types to include (inject action)")]
    pub observation_types: Option<Vec<String>>,

    /// Maximum token budget for injected context.
    #[schemars(description = "Maximum token budget for injected context")]
    pub max_tokens: Option<usize>,

    /// Maximum results.
    #[schemars(description = "Maximum results")]
    pub limit: Option<u32>,
}

// =============================================================================
// Session Tool - Consolidates session_summary and agent_session tools (6 → 1)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    /// Create a new session.
    Create,
    /// Get an existing session.
    Get,
    /// Update an existing session.
    Update,
    /// List available sessions.
    List,
    /// Summarize a session.
    Summarize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct SessionArgs {
    /// Action: create, get, update, list, summarize.
    #[schemars(description = "Action: create, get, update, list, summarize")]
    pub action: SessionAction,

    /// Session ID (required for get, update, summarize).
    #[schemars(description = "Session ID (required for get, update, summarize)")]
    pub session_id: Option<String>,

    /// Data payload for create/update (JSON object).
    #[schemars(description = "Data payload for create/update (JSON object)")]
    pub data: Option<serde_json::Value>,

    /// Filter by project ID.
    #[schemars(description = "Filter by project ID")]
    pub project_id: Option<String>,

    /// Filter by agent type.
    #[schemars(description = "Filter by agent type")]
    pub agent_type: Option<String>,

    /// Filter by status.
    #[schemars(description = "Filter by status")]
    pub status: Option<String>,

    /// Maximum results for list.
    #[schemars(description = "Maximum results for list")]
    pub limit: Option<u32>,
}

// =============================================================================
// Agent Tool - Consolidates store_tool_call, store_delegation (2 → 1)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentAction {
    /// Log a tool execution.
    LogTool,
    /// Log a delegation event.
    LogDelegation,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct AgentArgs {
    /// Action: log_tool, log_delegation.
    #[schemars(description = "Action: log_tool, log_delegation")]
    pub action: AgentAction,

    /// Session ID for the agent.
    #[schemars(description = "Session ID for the agent")]
    #[validate(length(min = 1))]
    pub session_id: String,

    /// Activity data (JSON object with tool/delegation details).
    #[schemars(description = "Activity data (JSON object with tool/delegation details)")]
    pub data: serde_json::Value,
}

// =============================================================================
// VCS Tool - Consolidates VCS tools (index_vcs_repository, compare_branches,
// search_branch, list_repositories, analyze_impact)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VcsAction {
    /// List repositories in the workspace.
    ListRepositories,
    /// Index a repository.
    IndexRepository,
    /// Compare two branches.
    CompareBranches,
    /// Search within a branch.
    SearchBranch,
    /// Analyze impact of changes.
    AnalyzeImpact,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct VcsArgs {
    /// Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact.
    #[schemars(
        description = "Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact"
    )]
    pub action: VcsAction,

    /// Repository identifier.
    #[schemars(description = "Repository identifier")]
    pub repo_id: Option<String>,

    /// Repository path on disk.
    #[schemars(description = "Repository path on disk")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub repo_path: Option<String>,

    /// Base branch name.
    #[schemars(description = "Base branch name")]
    pub base_branch: Option<String>,

    /// Compare/target branch name.
    #[schemars(description = "Compare/target branch name")]
    pub target_branch: Option<String>,

    /// Search query for branch search.
    #[schemars(description = "Search query for branch search")]
    pub query: Option<String>,

    /// Branches to index (default: repo default branch).
    #[schemars(description = "Branches to index (default: repo default branch)")]
    pub branches: Option<Vec<String>>,

    /// Whether to include commit history when indexing.
    #[schemars(description = "Whether to include commit history when indexing")]
    pub include_commits: Option<bool>,

    /// Commit history depth (default: 50 from config, or 1000 if no config).
    #[schemars(
        description = "Commit history depth (default: 50 from config, or 1000 if no config)"
    )]
    pub depth: Option<usize>,

    /// Limit for search or list actions.
    #[schemars(description = "Limit for search or list actions")]
    pub limit: Option<u32>,
}
// =============================================================================
// Project Tool - Consolidates all project_* tools (9 tools → 1)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAction {
    /// Create a new resource.
    Create,
    /// Update an existing resource.
    Update,
    /// List resources.
    List,
    /// Delete a resource.
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectResource {
    /// Project phase.
    Phase,
    /// Project issue.
    Issue,
    /// Issue dependency.
    Dependency,
    /// Project decision.
    Decision,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct ProjectArgs {
    /// Action: create, update, list, delete.
    #[schemars(description = "Action: create, update, list, delete")]
    pub action: ProjectAction,

    /// Resource type: phase, issue, dependency, decision.
    #[schemars(description = "Resource type: phase, issue, dependency, decision")]
    pub resource: ProjectResource,

    /// Project ID.
    #[schemars(description = "Project ID")]
    pub project_id: String,

    /// Data payload for create/update (JSON object).
    #[schemars(description = "Data payload for create/update (JSON object)")]
    pub data: Option<serde_json::Value>,

    /// Additional filters for list action.
    #[schemars(description = "Additional filters for list action")]
    pub filters: Option<serde_json::Value>,
}
