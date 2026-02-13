// =============================================================================
// TODO(REF004): File too large (965 lines). Split into smaller modules (max 500 lines).
// =============================================================================

use mcb_domain::value_objects::ids::SessionId;
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
    /// Start git-aware incremental indexing.
    GitIndex,
    /// Get the status of current indexing operation.
    Status,
    /// Clear the index for a collection.
    Clear,
}

/// Arguments for the index tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct IndexArgs {
    /// Action to perform: start, git_index, status, clear.
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
    /// Search across context snapshots.
    Context,
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

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Collection name.
    #[schemars(description = "Collection name", with = "String")]
    pub collection: Option<String>,

    /// File extensions to include (code search only).
    #[schemars(
        description = "File extensions to include (code search only)",
        with = "Vec<String>"
    )]
    pub extensions: Option<Vec<String>>,

    /// Additional search filters.
    #[schemars(description = "Additional search filters", with = "Vec<String>")]
    pub filters: Option<Vec<String>>,

    /// Maximum results to return.
    #[schemars(description = "Maximum results to return", with = "u32")]
    pub limit: Option<u32>,

    /// Minimum similarity score (0.0-1.0).
    #[schemars(description = "Minimum similarity score (0.0-1.0)", with = "f32")]
    #[validate(range(min = 0.0, max = 1.0, message = "Min score must be 0.0-1.0"))]
    pub min_score: Option<f32>,

    /// Filter by tags (for memory search).
    #[schemars(
        description = "Filter by tags (for memory search)",
        with = "Vec<String>"
    )]
    pub tags: Option<Vec<String>>,

    /// Filter by session ID (for memory search).
    #[schemars(
        description = "Filter by session ID (for memory search)",
        with = "SessionId"
    )]
    pub session_id: Option<SessionId>,

    /// JWT token for authenticated requests.
    #[schemars(description = "JWT token for authenticated requests", with = "String")]
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
// TODO(REF002): Duplicate definition 'ValidateArgs' also found in 'crates/mcb/src/cli/validate.rs'.
// Consider consolidating to a common crate or shared module.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct ValidateArgs {
    /// Action: run (validate), list_rules, analyze (complexity).
    #[schemars(description = "Action: run (validate), list_rules, analyze (complexity)")]
    pub action: ValidateAction,

    /// Scope: file or project.
    #[schemars(description = "Scope: file or project", with = "ValidateScope")]
    pub scope: Option<ValidateScope>,

    /// Path to file or project directory.
    #[schemars(description = "Path to file or project directory", with = "String")]
    pub path: Option<String>,

    /// Specific rules to run (empty = all).
    #[schemars(
        description = "Specific rules to run (empty = all)",
        with = "Vec<String>"
    )]
    pub rules: Option<Vec<String>>,

    /// Rule category filter.
    #[schemars(description = "Rule category filter", with = "String")]
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
// TODO(KISS001): Struct MemoryArgs has too many fields (18 fields, max: 16).
// Split into smaller structs or use composition.
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

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Data payload for store actions (JSON object).
    #[schemars(
        description = "Data payload for store action. observation: {content, type?, tags?, metadata?}; execution: {command, output?, status?}; quality_gate: {gate_name, status, details?}; error_pattern: {error_type, message, fix?}; session: {session_id, topics?, decisions?, next_steps?, key_files?}",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    /// Resource IDs for get action.
    #[schemars(description = "Resource IDs for get action", with = "Vec<String>")]
    pub ids: Option<Vec<String>>,

    /// Filter by project ID.
    #[schemars(description = "Filter by project ID", with = "String")]
    pub project_id: Option<String>,

    /// Filter by repository ID.
    #[schemars(description = "Filter by repository ID", with = "String")]
    pub repo_id: Option<String>,

    /// Filter by session ID.
    #[schemars(description = "Filter by session ID", with = "SessionId")]
    pub session_id: Option<SessionId>,

    /// Filter by parent session ID.
    #[schemars(description = "Filter by parent session ID", with = "String")]
    pub parent_session_id: Option<String>,

    /// Filter by tags.
    #[schemars(description = "Filter by tags", with = "Vec<String>")]
    pub tags: Option<Vec<String>>,

    /// Query string for list/search actions.
    #[schemars(description = "Query string for list/search actions", with = "String")]
    pub query: Option<String>,

    /// Anchor observation ID (for timeline action).
    #[schemars(
        description = "Anchor observation ID (for timeline action)",
        with = "String"
    )]
    pub anchor_id: Option<String>,

    /// Timeline depth before the anchor (default: 5).
    #[schemars(
        description = "Timeline depth before the anchor (default: 5)",
        with = "usize"
    )]
    pub depth_before: Option<usize>,

    /// Timeline depth after the anchor (default: 5).
    #[schemars(
        description = "Timeline depth after the anchor (default: 5)",
        with = "usize"
    )]
    pub depth_after: Option<usize>,

    /// Time window in seconds (for timeline action).
    #[schemars(
        description = "Time window in seconds (for timeline action)",
        with = "i64"
    )]
    pub window_secs: Option<i64>,

    /// Observation types to include (inject action).
    #[schemars(
        description = "Observation types to include (inject action)",
        with = "Vec<String>"
    )]
    pub observation_types: Option<Vec<String>>,

    /// Maximum token budget for injected context.
    #[schemars(
        description = "Maximum token budget for injected context",
        with = "usize"
    )]
    pub max_tokens: Option<usize>,

    /// Maximum results.
    #[schemars(description = "Maximum results", with = "u32")]
    pub limit: Option<u32>,
}

// =============================================================================
// Session Tool - Consolidates session_summary and agent_session tools (6 → 1)
// =============================================================================

/// Actions available for session management operations
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

/// Arguments for session management tool operations
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct SessionArgs {
    /// Action: create, get, update, list, summarize.
    #[schemars(description = "Action: create, get, update, list, summarize")]
    pub action: SessionAction,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Session ID (required for get, update, summarize).
    #[schemars(
        description = "Session ID (required for get, update, summarize)",
        with = "SessionId"
    )]
    pub session_id: Option<SessionId>,

    /// Data payload for create/update (JSON object).
    #[schemars(
        description = "Data payload for create/update. create requires model and accepts session_summary_id?, agent_type? (or top-level args.agent_type), parent_session_id?, prompt_summary?, project_id?, worktree_id?; update accepts mutable session fields",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    /// Filter by project ID.
    #[schemars(description = "Filter by project ID", with = "String")]
    pub project_id: Option<String>,

    /// Filter by worktree ID.
    #[schemars(description = "Filter by worktree ID", with = "String")]
    pub worktree_id: Option<String>,

    /// Filter by parent session ID.
    #[schemars(description = "Filter by parent session ID", with = "String")]
    pub parent_session_id: Option<String>,

    /// Filter by agent type.
    #[schemars(description = "Filter by agent type", with = "String")]
    pub agent_type: Option<String>,

    /// Filter by status.
    #[schemars(description = "Filter by status", with = "String")]
    pub status: Option<String>,

    /// Maximum results for list.
    #[schemars(description = "Maximum results for list", with = "u32")]
    pub limit: Option<u32>,
}

// =============================================================================
// Agent Tool - Consolidates store_tool_call, store_delegation (2 → 1)
// =============================================================================

/// Actions available for agent activity logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentAction {
    /// Log a tool execution.
    LogTool,
    /// Log a delegation event.
    LogDelegation,
}

/// Arguments for agent activity logging operations
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct AgentArgs {
    /// Action: log_tool, log_delegation.
    #[schemars(description = "Action: log_tool, log_delegation")]
    pub action: AgentAction,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Session ID for the agent.
    #[schemars(description = "Session ID for the agent")]
    pub session_id: SessionId,

    /// Activity data (JSON object with tool/delegation details).
    #[schemars(
        description = "Activity data payload. log_tool: {tool_name, params_summary?, success, error_message?, duration_ms?}; log_delegation: {child_session_id, prompt, prompt_embedding_id?, result?, success, duration_ms?}"
    )]
    pub data: serde_json::Value,
}

// =============================================================================
// VCS Tool - Consolidates VCS tools (index_vcs_repository, compare_branches,
// search_branch, list_repositories, analyze_impact)
// =============================================================================

/// Actions available for version control system operations
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

/// Arguments for version control system operations
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct VcsArgs {
    /// Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact.
    #[schemars(
        description = "Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact"
    )]
    pub action: VcsAction,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Repository identifier.
    #[schemars(description = "Repository identifier", with = "String")]
    pub repo_id: Option<String>,

    /// Repository path on disk.
    #[schemars(description = "Repository path on disk", with = "String")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub repo_path: Option<String>,

    /// Base branch name.
    #[schemars(description = "Base branch name", with = "String")]
    pub base_branch: Option<String>,

    /// Compare/target branch name.
    #[schemars(description = "Compare/target branch name", with = "String")]
    pub target_branch: Option<String>,

    /// Search query for branch search.
    #[schemars(description = "Search query for branch search", with = "String")]
    pub query: Option<String>,

    /// Branches to index (default: repo default branch).
    #[schemars(
        description = "Branches to index (default: repo default branch)",
        with = "Vec<String>"
    )]
    pub branches: Option<Vec<String>>,

    /// Whether to include commit history when indexing.
    #[schemars(
        description = "Whether to include commit history when indexing",
        with = "bool"
    )]
    pub include_commits: Option<bool>,

    /// Commit history depth (default: 50 from config, or 1000 if no config).
    #[schemars(
        description = "Commit history depth (default: 50 from config, or 1000 if no config)",
        with = "usize"
    )]
    pub depth: Option<usize>,

    /// Limit for search or list actions.
    #[schemars(description = "Limit for search or list actions", with = "u32")]
    pub limit: Option<u32>,
}

// =============================================================================
// Entity Tool - Consolidates VCS/Plan/Issue/Org entity CRUD (4 → 1)
// =============================================================================

/// CRUD actions for entity resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityAction {
    /// Create a new entity.
    Create,
    /// Get an entity by ID.
    Get,
    /// Update an existing entity.
    Update,
    /// List entities matching criteria.
    List,
    /// Delete an entity by ID.
    Delete,
    /// Release an assignment (VCS assignment only).
    Release,
}

/// Target resource type for consolidated entity operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityResource {
    /// VCS repository resource.
    Repository,
    /// VCS branch resource.
    Branch,
    /// VCS worktree resource.
    Worktree,
    /// VCS assignment resource.
    Assignment,
    /// Plan resource.
    Plan,
    /// Plan version resource.
    Version,
    /// Plan review resource.
    Review,
    /// Issue resource.
    Issue,
    /// Issue comment resource.
    Comment,
    /// Issue label resource.
    Label,
    /// Issue label assignment resource.
    LabelAssignment,
    /// Organization resource.
    Org,
    /// User resource.
    User,
    /// Team resource.
    Team,
    /// Team member resource.
    TeamMember,
    /// API key resource.
    ApiKey,
}

/// Arguments for the consolidated `entity` MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct EntityArgs {
    /// CRUD action to perform.
    pub action: EntityAction,
    /// Target resource type.
    pub resource: EntityResource,
    /// JSON payload for create/update actions.
    #[schemars(with = "serde_json::Value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Resource ID (for get/update/delete/release).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Organization ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// Project ID (project-scoped list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// Repository ID (branch/worktree list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<String>,
    /// Worktree ID (assignment list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_id: Option<String>,
    /// Plan ID (version list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    /// Plan version ID (review list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_version_id: Option<String>,
    /// Issue ID (comment/list/label assignment operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    /// Label ID (label unassignment operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_id: Option<String>,
    /// Team ID (team member list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// User ID (team member delete operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// User email (lookup operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// =============================================================================
// VCS Entity Tool - Repository, Branch, Worktree, Assignment CRUD
// =============================================================================

/// CRUD actions for VCS entity resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VcsEntityAction {
    /// Create a new entity.
    Create,
    /// Get an entity by ID.
    Get,
    /// Update an existing entity.
    Update,
    /// List entities matching criteria.
    List,
    /// Delete an entity by ID.
    Delete,
    /// Release an assignment.
    Release,
}

/// Target resource type for VCS entity operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VcsEntityResource {
    /// Repository resource.
    Repository,
    /// Branch resource.
    Branch,
    /// Worktree resource.
    Worktree,
    /// Agent-worktree assignment resource.
    Assignment,
}

/// Arguments for the consolidated `vcs_entity` MCP tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct VcsEntityArgs {
    /// CRUD action to perform.
    #[schemars(description = "Action: create, get, update, list, delete, release")]
    pub action: VcsEntityAction,

    /// Target resource type.
    #[schemars(description = "Resource: repository, branch, worktree, assignment")]
    pub resource: VcsEntityResource,

    /// Resource ID (for get/update/delete/release).
    #[schemars(description = "Resource ID (for get/update/delete/release)")]
    pub id: Option<String>,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Project ID (for repository listing).
    #[schemars(description = "Project ID (for repository listing)")]
    pub project_id: Option<String>,

    /// Repository ID (for branch/worktree listing).
    #[schemars(description = "Repository ID (for branch/worktree listing)")]
    pub repository_id: Option<String>,

    /// Worktree ID (for assignment listing).
    #[schemars(description = "Worktree ID (for assignment listing)")]
    pub worktree_id: Option<String>,

    /// Data payload for create/update (JSON object).
    #[schemars(
        description = "Data payload for create/update. phase: {name, status, order}; issue: {title, description?, status?, priority?}; dependency: {from_issue_id, to_issue_id, kind?}; decision: {title, rationale, impact?, status?}",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}

/// CRUD actions for plan entity resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntityAction {
    /// Create a new entity.
    Create,
    /// Get an entity by ID.
    Get,
    /// Update an existing entity.
    Update,
    /// List entities matching criteria.
    List,
    /// Delete an entity by ID.
    Delete,
}

/// Target resource type for plan entity operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntityResource {
    /// Plan resource.
    Plan,
    /// Plan version resource.
    Version,
    /// Plan review resource.
    Review,
}

/// Arguments for the consolidated `plan_entity` MCP tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct PlanEntityArgs {
    /// CRUD action to perform.
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: PlanEntityAction,

    /// Target resource type.
    #[schemars(description = "Resource: plan, version, review")]
    pub resource: PlanEntityResource,

    /// Resource ID (for get/update/delete).
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Project ID (for plan listing).
    #[schemars(description = "Project ID (for plan listing)")]
    pub project_id: Option<String>,

    /// Plan ID (for version listing).
    #[schemars(description = "Plan ID (for version listing)")]
    pub plan_id: Option<String>,

    /// Plan version ID (for review listing).
    #[schemars(description = "Plan version ID (for review listing)")]
    pub plan_version_id: Option<String>,

    /// Data payload for create/update (JSON object).
    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}

/// Actions for org entity resource management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrgEntityAction {
    /// Create a new entity.
    Create,
    /// Get an entity by identifier.
    Get,
    /// Update an existing entity.
    Update,
    /// List entities matching filters.
    List,
    /// Delete an entity by identifier.
    Delete,
}

/// Types of org entity resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrgEntityResource {
    /// Organization resource.
    Org,
    /// User resource.
    User,
    /// Team resource.
    Team,
    /// Team-member link resource.
    TeamMember,
    /// API key resource.
    ApiKey,
}

/// Arguments for the consolidated `org_entity` MCP tool
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct OrgEntityArgs {
    /// Action to perform.
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: OrgEntityAction,
    /// Resource type to target.
    #[schemars(description = "Resource: org, user, team, team_member, api_key")]
    pub resource: OrgEntityResource,
    /// Resource ID for get/update/delete operations.
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,
    /// Organization ID for list operations.
    #[schemars(description = "Organization ID (for listing users/teams/api_keys)")]
    pub org_id: Option<String>,
    /// Team ID used by team-member list/delete.
    #[schemars(description = "Team ID (for listing members)")]
    pub team_id: Option<String>,
    /// User ID used by team-member delete.
    #[schemars(description = "User ID (for removing team member)")]
    pub user_id: Option<String>,
    /// Email used for user lookup when ID is omitted.
    #[schemars(description = "Email (for user lookup by email)")]
    pub email: Option<String>,
    /// JSON payload for create/update actions.
    #[schemars(description = "Data payload for create/update (JSON object)")]
    #[schemars(with = "serde_json::Value")]
    pub data: Option<serde_json::Value>,
}

/// CRUD actions for issue entity resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssueEntityAction {
    /// Create a new entity.
    Create,
    /// Get an entity by ID.
    Get,
    /// Update an existing entity.
    Update,
    /// List entities matching criteria.
    List,
    /// Delete an entity by ID.
    Delete,
}

/// Target resource type for issue entity operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssueEntityResource {
    /// Issue resource.
    Issue,
    /// Comment resource.
    Comment,
    /// Label resource.
    Label,
    /// Label assignment resource.
    LabelAssignment,
}

/// Arguments for the consolidated `issue_entity` MCP tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
pub struct IssueEntityArgs {
    /// CRUD action to perform.
    #[schemars(description = "Action: create, get, update, list, delete")]
    pub action: IssueEntityAction,

    /// Target resource type.
    #[schemars(description = "Resource: issue, comment, label, label_assignment")]
    pub resource: IssueEntityResource,

    /// Resource ID (for get/update/delete).
    #[schemars(description = "Resource ID (for get/update/delete)")]
    pub id: Option<String>,

    /// Organization ID (uses default if omitted).
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Project ID (for issue/label listing).
    #[schemars(description = "Project ID (for issue/label listing)")]
    pub project_id: Option<String>,

    /// Issue ID (for comment listing and label assignments).
    #[schemars(description = "Issue ID (for comment listing and label assignments)")]
    pub issue_id: Option<String>,

    /// Label ID (for label unassignment).
    #[schemars(description = "Label ID (for label unassignment)")]
    pub label_id: Option<String>,

    /// Data payload for create/update (JSON object).
    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,
}

// =============================================================================
// Project Tool - Consolidates all project_* tools (9 tools → 1)
// =============================================================================

/// Actions available for project resource management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAction {
    /// Create a new resource.
    Create,
    /// Get an existing resource.
    Get,
    /// Update an existing resource.
    Update,
    /// List resources.
    List,
    /// Delete a resource.
    Delete,
}

/// Types of project resources that can be managed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectResource {
    /// Project metadata.
    Project,
    /// Project phase.
    Phase,
    /// Project issue.
    Issue,
    /// Issue dependency.
    Dependency,
    /// Project decision.
    Decision,
}

/// Arguments for project resource management operations
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
    #[schemars(
        description = "Data payload for create/update (JSON object)",
        with = "serde_json::Value"
    )]
    pub data: Option<serde_json::Value>,

    /// Additional filters for list action.
    #[schemars(
        description = "Additional filters for list action",
        with = "serde_json::Value"
    )]
    pub filters: Option<serde_json::Value>,
}
