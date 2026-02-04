//! Memory tool argument types (timeline, observations, search, inject context).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

fn default_timeline_depth() -> usize {
    5
}

/// Arguments for the `memory_timeline` tool (Step 2 of progressive disclosure).
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[rustfmt::skip]
#[schemars(description = "[EXPERIMENTAL] Get context around a specific observation in chronological order")]
pub struct MemoryTimelineArgs {
    #[schemars(description = "Anchor observation ID to center the timeline around")]
    pub anchor_id: Option<String>,

    #[schemars(description = "Query to find anchor automatically (if anchor_id not provided)")]
    pub query: Option<String>,

    #[serde(default = "default_timeline_depth")]
    #[validate(range(min = 0, max = 50))]
    #[schemars(description = "Number of observations before anchor (default: 5)")]
    pub depth_before: usize,

    #[serde(default = "default_timeline_depth")]
    #[validate(range(min = 0, max = 50))]
    #[schemars(description = "Number of observations after anchor (default: 5)")]
    pub depth_after: usize,

    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,

    #[schemars(description = "Filter by observation type")]
    pub observation_type: Option<String>,
}

/// Arguments for the `memory_get_observations` tool (Step 3 of progressive disclosure).
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "[EXPERIMENTAL] Fetch full details for specific observation IDs")]
pub struct MemoryGetObservationsArgs {
    #[validate(length(min = 1, message = "At least one ID is required"))]
    #[schemars(description = "Array of observation IDs to fetch")]
    pub ids: Vec<String>,
}

/// Arguments for the `memory_search` tool (token-efficient index - Step 1 of 3).
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[rustfmt::skip]
#[schemars(description = "Token-efficient memory search returning index only (IDs, types, scores, previews). Use memory_get_observations with returned IDs for full details.")]
pub struct MemorySearchArgs {
    #[validate(length(min = 1, max = 1000, message = "Query must be 1-1000 chars"))]
    #[schemars(description = "Search query for semantic matching")]
    pub query: String,

    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schemars(description = "Maximum number of results (default: 10)")]
    pub limit: usize,

    #[schemars(description = "Filter by tags")]
    pub tags: Option<Vec<String>>,

    #[schemars(
        description = "Filter by observation type: code, decision, context, error, summary, execution, quality_gate"
    )]
    pub observation_type: Option<String>,

    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,
}

/// Arguments for the `memory_store_execution` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Store execution results in semantic memory")]
pub struct MemoryStoreExecutionArgs {
    #[validate(length(min = 1, max = 2000, message = "Command must be 1-2000 chars"))]
    #[schemars(description = "Command executed (full command string)")]
    pub command: String,

    #[schemars(description = "Exit code from execution")]
    pub exit_code: Option<i32>,

    #[schemars(description = "Execution duration in milliseconds")]
    pub duration_ms: Option<i64>,

    #[schemars(description = "Whether the execution succeeded")]
    pub success: bool,

    #[schemars(description = "Execution type: test, lint, build, ci")]
    pub execution_type: String,

    #[schemars(description = "Coverage percentage (0-100)")]
    pub coverage: Option<f32>,

    #[serde(default)]
    #[schemars(description = "Files affected by the execution")]
    pub files_affected: Vec<String>,

    #[schemars(description = "Summary of output (truncated or aggregated)")]
    pub output_summary: Option<String>,

    #[schemars(description = "Number of warnings emitted")]
    pub warnings_count: Option<i32>,

    #[schemars(description = "Number of errors emitted")]
    pub errors_count: Option<i32>,

    #[schemars(description = "Session ID to associate with this execution")]
    pub session_id: Option<String>,

    #[schemars(description = "Repository ID for context")]
    pub repo_id: Option<String>,

    #[schemars(description = "VCS branch related to this execution")]
    pub branch: Option<String>,

    #[schemars(description = "VCS commit related to this execution")]
    pub commit: Option<String>,
}

/// Arguments for the `memory_get_executions` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Retrieve execution history with optional filters")]
pub struct MemoryGetExecutionsArgs {
    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[schemars(description = "Maximum number of executions to return (default: 10)")]
    pub limit: usize,

    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,

    #[schemars(description = "Filter by execution type: test, lint, build, ci")]
    pub execution_type: Option<String>,

    #[schemars(description = "Filter by success status")]
    pub success: Option<bool>,

    #[schemars(description = "Filter by branch")]
    pub branch: Option<String>,

    #[schemars(description = "Filter by commit")]
    pub commit: Option<String>,

    #[schemars(description = "Filter by start timestamp (inclusive)")]
    pub start_time: Option<i64>,

    #[schemars(description = "Filter by end timestamp (inclusive)")]
    pub end_time: Option<i64>,
}

/// Arguments for the `memory_store_quality_gate` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Store quality gate results in semantic memory")]
pub struct MemoryStoreQualityGateArgs {
    #[validate(length(min = 1, max = 200, message = "Gate name must be 1-200 chars"))]
    #[schemars(description = "Quality gate name")]
    pub gate_name: String,

    #[schemars(description = "Quality gate status: passed, failed, warning, skipped")]
    pub status: String,

    #[schemars(description = "Optional message or notes for this gate")]
    pub message: Option<String>,

    #[schemars(description = "Timestamp for the gate result (RFC3339)")]
    pub timestamp: DateTime<Utc>,

    #[schemars(description = "Optional execution ID related to this gate")]
    pub execution_id: Option<String>,

    #[schemars(description = "Session ID to associate with this quality gate")]
    pub session_id: Option<String>,

    #[schemars(description = "Repository ID for context")]
    pub repo_id: Option<String>,

    #[schemars(description = "VCS branch related to this quality gate")]
    pub branch: Option<String>,

    #[schemars(description = "VCS commit related to this quality gate")]
    pub commit: Option<String>,
}

/// Arguments for the `memory_get_quality_gates` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Retrieve quality gate results with optional filters")]
pub struct MemoryGetQualityGatesArgs {
    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[schemars(description = "Maximum number of gate results to return (default: 10)")]
    pub limit: usize,

    #[schemars(description = "Filter by session ID")]
    pub session_id: Option<String>,

    #[schemars(description = "Filter by repository ID")]
    pub repo_id: Option<String>,

    #[schemars(description = "Filter by gate name")]
    pub gate_name: Option<String>,

    #[schemars(description = "Filter by status: passed, failed, warning, skipped")]
    pub status: Option<String>,

    #[schemars(description = "Filter by execution ID")]
    pub execution_id: Option<String>,

    #[schemars(description = "Filter by branch")]
    pub branch: Option<String>,

    #[schemars(description = "Filter by commit")]
    pub commit: Option<String>,

    #[schemars(description = "Filter by start timestamp (inclusive)")]
    pub start_time: Option<i64>,

    #[schemars(description = "Filter by end timestamp (inclusive)")]
    pub end_time: Option<i64>,
}

/// Arguments for the `memory_record_error_pattern` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Record a new error pattern or match an existing one")]
pub struct MemoryRecordErrorPatternArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID for this error pattern")]
    pub project_id: String,

    #[validate(length(
        min = 1,
        max = 4000,
        message = "pattern_signature must be 1-4000 chars"
    ))]
    #[schemars(description = "Normalized error signature for pattern matching")]
    pub pattern_signature: String,

    #[validate(length(min = 1, max = 4000, message = "description must be 1-4000 chars"))]
    #[schemars(description = "Human-readable description of the error pattern")]
    pub description: String,

    #[validate(length(min = 1, message = "category cannot be empty"))]
    #[schemars(
        description = "Error pattern category: compilation, runtime, test, lint, build, config, network, other"
    )]
    pub category: String,

    #[serde(default)]
    #[schemars(description = "Known solutions or remediation steps")]
    pub solutions: Vec<String>,

    #[serde(default)]
    #[schemars(description = "Files affected by this error pattern")]
    pub affected_files: Vec<String>,

    #[serde(default)]
    #[schemars(description = "Tags for categorizing the pattern")]
    pub tags: Vec<String>,

    #[schemars(description = "Observation ID for the error occurrence")]
    pub observation_id: Option<String>,

    #[validate(range(min = 0, max = 1000))]
    #[schemars(description = "Match confidence scaled by 1000 (e.g., 950 = 0.95)")]
    pub confidence: Option<i64>,

    #[schemars(description = "Index of the solution applied (if any)")]
    pub solution_applied: Option<i32>,

    #[schemars(description = "Whether the resolution was successful")]
    pub resolution_successful: Option<bool>,

    #[schemars(description = "Timestamp when the pattern was matched (epoch seconds)")]
    pub matched_at: Option<i64>,

    #[schemars(description = "Timestamp when the issue was resolved (epoch seconds)")]
    pub resolved_at: Option<i64>,
}

/// Arguments for the `memory_get_error_patterns` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Retrieve error patterns with optional filters")]
pub struct MemoryGetErrorPatternsArgs {
    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[schemars(description = "Maximum number of patterns to return (default: 10)")]
    pub limit: usize,

    #[schemars(description = "Filter by project ID")]
    pub project_id: Option<String>,

    #[schemars(
        description = "Filter by category: compilation, runtime, test, lint, build, config, network, other"
    )]
    pub category: Option<String>,

    #[schemars(description = "Filter by tags")]
    pub tags: Option<Vec<String>>,

    #[schemars(description = "Filter by normalized pattern signature")]
    pub pattern_signature: Option<String>,

    #[schemars(description = "Filter by affected files")]
    pub affected_files: Option<Vec<String>>,

    #[schemars(description = "Filter by minimum occurrence count")]
    pub min_occurrence_count: Option<i64>,

    #[schemars(description = "Filter by maximum occurrence count")]
    pub max_occurrence_count: Option<i64>,

    #[schemars(description = "Filter by start timestamp (inclusive)")]
    pub start_time: Option<i64>,

    #[schemars(description = "Filter by end timestamp (inclusive)")]
    pub end_time: Option<i64>,
}

/// Arguments for the `memory_inject_context` tool (SessionStart hook integration).
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "[EXPERIMENTAL] Generate context bundle for session start injection")]
pub struct MemoryInjectContextArgs {
    #[validate(length(min = 1, message = "session_id cannot be empty"))]
    #[schemars(description = "Current session ID for context continuity")]
    pub session_id: String,

    #[schemars(description = "Repository ID for project-specific context")]
    pub repo_id: Option<String>,

    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 50))]
    #[schemars(description = "Maximum observations to include (default: 10)")]
    pub limit: usize,

    #[serde(default)]
    #[schemars(description = "Observation types to include (default: all)")]
    pub observation_types: Vec<String>,

    #[schemars(description = "Maximum token budget for context (approximate)")]
    pub max_tokens: Option<usize>,
}
