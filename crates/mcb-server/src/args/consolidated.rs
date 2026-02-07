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
    #[schemars(
        description = "Action to perform: start (index), status (check progress), clear (remove index)"
    )]
    pub action: IndexAction,

    #[schemars(description = "Path to codebase directory (required for 'start' action)")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub path: Option<String>,

    #[schemars(description = "Collection name for the index")]
    #[validate(custom(
        function = "super::validate_collection_name",
        message = "Invalid collection name"
    ))]
    pub collection: Option<String>,

    #[schemars(description = "File extensions to include (for 'start' action)")]
    pub extensions: Option<Vec<String>>,

    #[schemars(description = "Directories to exclude (for 'start' action)")]
    pub exclude_dirs: Option<Vec<String>>,

    #[schemars(description = "Glob patterns for files/directories to exclude")]
    pub ignore_patterns: Option<Vec<String>>,

    #[schemars(description = "Maximum file size to index (bytes)")]
    pub max_file_size: Option<u64>,

    #[schemars(description = "Follow symbolic links during indexing")]
    pub follow_symlinks: Option<bool>,

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
    #[schemars(description = "Natural language search query")]
    #[validate(length(min = 1))]
    pub query: String,

    #[schemars(description = "Resource to search: code or memory")]
    pub resource: SearchResource,

    #[schemars(description = "Collection name")]
    pub collection: Option<String>,

    #[schemars(description = "File extensions to include (code search only)")]
    pub extensions: Option<Vec<String>>,

    #[schemars(description = "Additional search filters")]
    pub filters: Option<Vec<String>>,

    #[schemars(description = "Maximum results to return")]
    pub limit: Option<u32>,

    #[schemars(description = "Minimum similarity score (0.0-1.0)")]
    #[validate(range(min = 0.0, max = 1.0, message = "Min score must be 0.0-1.0"))]
    pub min_score: Option<f32>,

    #[schemars(description = "Filter by tags (for memory search)")]
    pub tags: Option<Vec<String>>,

    #[schemars(description = "Filter by session ID (for memory search)")]
    pub session_id: Option<String>,

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
    #[schemars(description = "Action: run (validate), list_rules, analyze (complexity)")]
    pub action: ValidateAction,

    #[schemars(description = "Scope: file or project")]
    pub scope: Option<ValidateScope>,

    #[schemars(description = "Path to file or project directory")]
    pub path: Option<String>,

    #[schemars(description = "Specific rules to run (empty = all)")]
    pub rules: Option<Vec<String>>,

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
    #[schemars(description = "Action: store, get, list, timeline, inject")]
    pub action: MemoryAction,

    #[schemars(
        description = "Resource type: observation, execution, quality_gate, error_pattern, session"
    )]
    pub resource: MemoryResource,

    #[schemars(description = "Data payload for store actions (JSON object)")]
    pub data: Option<serde_json::Value>,

    #[schemars(description = "Resource IDs for get action")]
    pub ids: Option<Vec<String>>,

    #[schemars(description = "Filter by project ID")]
    pub project_id: Option<String>,

    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,

    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    #[schemars(description = "Filter by tags")]
    pub tags: Option<Vec<String>>,

    #[schemars(description = "Query string for list/search actions")]
    pub query: Option<String>,

    #[schemars(description = "Anchor observation ID (for timeline action)")]
    pub anchor_id: Option<String>,

    #[schemars(description = "Timeline depth before the anchor (default: 5)")]
    pub depth_before: Option<usize>,

    #[schemars(description = "Timeline depth after the anchor (default: 5)")]
    pub depth_after: Option<usize>,

    #[schemars(description = "Time window in seconds (for timeline action)")]
    pub window_secs: Option<i64>,

    #[schemars(description = "Observation types to include (inject action)")]
    pub observation_types: Option<Vec<String>>,

    #[schemars(description = "Maximum token budget for injected context")]
    pub max_tokens: Option<usize>,

    #[schemars(description = "Maximum results")]
    pub limit: Option<u32>,
}

// =============================================================================
// Session Tool - Consolidates session_summary and agent_session tools (6 → 1)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    Create,
    Get,
    Update,
    List,
    Summarize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct SessionArgs {
    #[schemars(description = "Action: create, get, update, list, summarize")]
    pub action: SessionAction,

    #[schemars(description = "Session ID (required for get, update, summarize)")]
    pub session_id: Option<String>,

    #[schemars(description = "Data payload for create/update (JSON object)")]
    pub data: Option<serde_json::Value>,

    #[schemars(description = "Filter by project ID")]
    pub project_id: Option<String>,

    #[schemars(description = "Filter by agent type")]
    pub agent_type: Option<String>,

    #[schemars(description = "Filter by status")]
    pub status: Option<String>,

    #[schemars(description = "Maximum results for list")]
    pub limit: Option<u32>,
}

// =============================================================================
// Agent Tool - Consolidates store_tool_call, store_delegation (2 → 1)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentAction {
    LogTool,
    LogDelegation,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct AgentArgs {
    #[schemars(description = "Action: log_tool, log_delegation")]
    pub action: AgentAction,

    #[schemars(description = "Session ID for the agent")]
    #[validate(length(min = 1))]
    pub session_id: String,

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
    ListRepositories,
    IndexRepository,
    CompareBranches,
    SearchBranch,
    AnalyzeImpact,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct VcsArgs {
    #[schemars(
        description = "Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact"
    )]
    pub action: VcsAction,

    #[schemars(description = "Repository identifier")]
    pub repo_id: Option<String>,

    #[schemars(description = "Repository path on disk")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub repo_path: Option<String>,

    #[schemars(description = "Base branch name")]
    pub base_branch: Option<String>,

    #[schemars(description = "Compare/target branch name")]
    pub target_branch: Option<String>,

    #[schemars(description = "Search query for branch search")]
    pub query: Option<String>,

    #[schemars(description = "Branches to index (default: repo default branch)")]
    pub branches: Option<Vec<String>>,

    #[schemars(description = "Whether to include commit history when indexing")]
    pub include_commits: Option<bool>,

    #[schemars(
        description = "Commit history depth (default: 50 from config, or 1000 if no config)"
    )]
    pub depth: Option<usize>,

    #[schemars(description = "Limit for search or list actions")]
    pub limit: Option<u32>,
}
// =============================================================================
// Project Tool - Consolidates all project_* tools (9 tools → 1)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAction {
    Create,
    Update,
    List,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectResource {
    Phase,
    Issue,
    Dependency,
    Decision,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct ProjectArgs {
    #[schemars(description = "Action: create, update, list, delete")]
    pub action: ProjectAction,

    #[schemars(description = "Resource type: phase, issue, dependency, decision")]
    pub resource: ProjectResource,

    #[schemars(description = "Project ID")]
    pub project_id: String,

    #[schemars(description = "Data payload for create/update (JSON object)")]
    pub data: Option<serde_json::Value>,

    #[schemars(description = "Additional filters for list action")]
    pub filters: Option<serde_json::Value>,
}
