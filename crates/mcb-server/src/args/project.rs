//! Project workflow state tool argument types (ADR-032).

use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

/// Arguments for the `project_create_phase` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Create a new project phase")]
pub struct ProjectCreatePhaseArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID that owns the phase")]
    pub project_id: String,

    #[validate(length(min = 1, message = "name cannot be empty"))]
    #[schemars(description = "Phase name")]
    pub name: String,

    #[validate(length(min = 1, message = "description cannot be empty"))]
    #[schemars(description = "Phase description")]
    pub description: String,

    #[validate(range(min = 1, message = "sequence must be >= 1"))]
    #[schemars(description = "Phase order in the roadmap (1-indexed)")]
    pub sequence: i32,

    #[schemars(description = "Phase status: planned, in_progress, blocked, completed, skipped")]
    pub status: String,

    #[schemars(description = "Started timestamp (epoch seconds)")]
    pub started_at: Option<i64>,

    #[schemars(description = "Completed timestamp (epoch seconds)")]
    pub completed_at: Option<i64>,
}

/// Arguments for the `project_update_phase` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Update a project phase")]
pub struct ProjectUpdatePhaseArgs {
    #[validate(length(min = 1, message = "id cannot be empty"))]
    #[schemars(description = "Phase ID to update")]
    pub id: String,

    #[schemars(description = "Updated phase name")]
    pub name: Option<String>,

    #[schemars(description = "Updated phase description")]
    pub description: Option<String>,

    #[validate(range(min = 1, message = "sequence must be >= 1"))]
    #[schemars(description = "Updated phase order in the roadmap (1-indexed)")]
    pub sequence: Option<i32>,

    #[schemars(description = "Updated status: planned, in_progress, blocked, completed, skipped")]
    pub status: Option<String>,

    #[schemars(description = "Updated started timestamp (epoch seconds)")]
    pub started_at: Option<i64>,

    #[schemars(description = "Updated completed timestamp (epoch seconds)")]
    pub completed_at: Option<i64>,
}

/// Arguments for the `project_list_phases` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "List phases for a project with optional filters")]
pub struct ProjectListPhasesArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID to list phases for")]
    pub project_id: String,

    #[schemars(
        description = "Filter by status: planned, in_progress, blocked, completed, skipped"
    )]
    pub status: Option<String>,

    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[schemars(description = "Maximum number of phases to return (default: 10)")]
    pub limit: usize,
}

/// Arguments for the `project_create_issue` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Create a new project issue")]
pub struct ProjectCreateIssueArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID that owns the issue")]
    pub project_id: String,

    #[schemars(description = "Optional phase ID for this issue")]
    pub phase_id: Option<String>,

    #[validate(length(min = 1, message = "title cannot be empty"))]
    #[schemars(description = "Issue title")]
    pub title: String,

    #[validate(length(min = 1, message = "description cannot be empty"))]
    #[schemars(description = "Issue description")]
    pub description: String,

    #[schemars(description = "Issue type: task, bug, feature, enhancement, documentation")]
    pub issue_type: String,

    #[schemars(description = "Issue status: open, in_progress, blocked, resolved, closed")]
    pub status: String,

    #[validate(range(min = 0, max = 4, message = "priority must be 0-4"))]
    #[schemars(description = "Issue priority (0=critical, 4=backlog)")]
    pub priority: i32,

    #[schemars(description = "Assignee user identifier")]
    pub assignee: Option<String>,

    #[serde(default)]
    #[schemars(description = "Labels for categorizing the issue")]
    pub labels: Vec<String>,
}

/// Arguments for the `project_update_issue` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Update a project issue")]
pub struct ProjectUpdateIssueArgs {
    #[validate(length(min = 1, message = "id cannot be empty"))]
    #[schemars(description = "Issue ID to update")]
    pub id: String,

    #[schemars(description = "Updated issue title")]
    pub title: Option<String>,

    #[schemars(description = "Updated issue description")]
    pub description: Option<String>,

    #[schemars(description = "Updated issue type: task, bug, feature, enhancement, documentation")]
    pub issue_type: Option<String>,

    #[schemars(description = "Updated issue status: open, in_progress, blocked, resolved, closed")]
    pub status: Option<String>,

    #[validate(range(min = 0, max = 4, message = "priority must be 0-4"))]
    #[schemars(description = "Updated issue priority (0=critical, 4=backlog)")]
    pub priority: Option<i32>,

    #[schemars(description = "Updated assignee user identifier")]
    pub assignee: Option<String>,

    #[schemars(description = "Updated labels for the issue")]
    pub labels: Option<Vec<String>>,
}

/// Arguments for the `project_list_issues` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "List issues for a project with filters")]
pub struct ProjectListIssuesArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID to list issues for")]
    pub project_id: String,

    #[schemars(description = "Filter by phase ID")]
    pub phase_id: Option<String>,

    #[schemars(
        description = "Filter by issue status: open, in_progress, blocked, resolved, closed"
    )]
    pub status: Option<String>,

    #[validate(range(min = 0, max = 4, message = "priority must be 0-4"))]
    #[schemars(description = "Filter by priority (0=critical, 4=backlog)")]
    pub priority: Option<i32>,

    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[schemars(description = "Maximum number of issues to return (default: 10)")]
    pub limit: usize,
}

/// Arguments for the `project_add_dependency` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Add a dependency between issues")]
pub struct ProjectAddDependencyArgs {
    #[validate(length(min = 1, message = "from_issue_id cannot be empty"))]
    #[schemars(description = "Source issue ID for the dependency")]
    pub from_issue_id: String,

    #[validate(length(min = 1, message = "to_issue_id cannot be empty"))]
    #[schemars(description = "Target issue ID for the dependency")]
    pub to_issue_id: String,

    #[schemars(description = "Dependency type: blocks, relates_to, duplicate_of, parent_of")]
    pub dependency_type: String,
}

/// Arguments for the `project_record_decision` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "Record a project decision")]
pub struct ProjectRecordDecisionArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID that owns the decision")]
    pub project_id: String,

    #[schemars(description = "Optional issue ID linked to this decision")]
    pub issue_id: Option<String>,

    #[validate(length(min = 1, message = "title cannot be empty"))]
    #[schemars(description = "Decision title")]
    pub title: String,

    #[validate(length(min = 1, message = "context cannot be empty"))]
    #[schemars(description = "Decision context")]
    pub context: String,

    #[validate(length(min = 1, message = "decision cannot be empty"))]
    #[schemars(description = "Decision made")]
    pub decision: String,

    #[validate(length(min = 1, message = "consequences cannot be empty"))]
    #[schemars(description = "Consequences or trade-offs")]
    pub consequences: String,
}

/// Arguments for the `project_list_decisions` tool.
#[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
#[schemars(description = "List decisions for a project")]
pub struct ProjectListDecisionsArgs {
    #[validate(length(min = 1, message = "project_id cannot be empty"))]
    #[schemars(description = "Project ID to list decisions for")]
    pub project_id: String,

    #[schemars(description = "Filter by issue ID")]
    pub issue_id: Option<String>,

    #[serde(default = "crate::args::default_limit")]
    #[validate(range(min = 1, max = 200))]
    #[schemars(description = "Maximum number of decisions to return (default: 10)")]
    pub limit: usize,
}
