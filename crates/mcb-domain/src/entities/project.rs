//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
//! Project Domain Entities
//!
//! # Overview
//! This module defines the core entities covering Project Management and Issue Tracking.
//! It includes the `Project` root entity, as well as `ProjectIssue`, `ProjectPhase`,
//! and related enums (`IssueType`, `IssueStatus`).
//!
//! # Key Concepts
//! - **Project**: A distinct codebase or module (e.g., a Rust crate, an NPM package).
//! - **Issue**: A unit of work (Task, Bug, Feature) tracked within a project.
//! - **Phase**: A milestone or stage in the project roadmap.
//! - **Dependency**: Directed relationships between issues (Blocks, `RelatesTo`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity! {
    /// Registered project in MCB - serves as root entity linking collections, observations, and file hashes.
    #[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
    pub struct Project { id, org_id, created_at, updated_at } {
        /// Display name of the project.
        pub name: String,
        /// Root filesystem path of the project.
        pub path: String,
    }
}

/// Project type detected from manifest files
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Represents a Rust project identified by Cargo.toml.
    Cargo {
        /// Name of the package from Cargo.toml.
        name: String,
        /// Version string from Cargo.toml.
        version: String,
        /// List of direct dependencies.
        dependencies: Vec<String>,
    },
    /// Represents a Node.js project identified by package.json.
    Npm {
        /// Name of the package from package.json.
        name: String,
        /// Version string from package.json.
        version: String,
        /// List of direct dependencies.
        dependencies: Vec<String>,
    },
    /// Represents a Python project identified by pyproject.toml or requirements.txt.
    Python {
        /// Name of the project (if available).
        name: String,
        /// Version string (if available).
        version: Option<String>,
        /// List of direct dependencies.
        dependencies: Vec<String>,
    },
    /// Represents a Go project identified by go.mod.
    Go {
        /// Module path from go.mod.
        module: String,
        /// Go version requirement.
        go_version: String,
        /// List of direct dependencies.
        dependencies: Vec<String>,
    },
    /// Represents a Maven project identified by pom.xml.
    Maven {
        /// Group ID of the artifact.
        group_id: String,
        /// Artifact ID.
        artifact_id: String,
        /// Version string.
        version: String,
        /// List of direct dependencies.
        dependencies: Vec<String>,
    },
}

impl ProjectType {
    /// Get the project name without needing helper functions.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Cargo { name, .. } | Self::Npm { name, .. } | Self::Python { name, .. } => name,
            Self::Go { module, .. } => module,
            Self::Maven { artifact_id, .. } => artifact_id,
        }
    }

    /// Get the normalized type label for the project.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Cargo { .. } => "cargo",
            Self::Npm { .. } => "npm",
            Self::Python { .. } => "python",
            Self::Go { .. } => "go",
            Self::Maven { .. } => "maven",
        }
    }
}

/// Detected project with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProject {
    /// Stable identifier for the detected project (could be UUID)
    pub id: String,
    /// Path relative to repository root
    pub path: String,
    /// Detected project type with metadata
    pub project_type: ProjectType,
    /// Parent repository ID (for submodule linking)
    pub parent_repo_id: Option<String>,
}

// ============================================================================
// Workflow State
// ============================================================================

crate::define_string_enum! {
    /// Represents the execution state of a project phase.
    pub enum PhaseStatus [strum = "snake_case"] {
        /// Indicates the phase is scheduled but hasn't started.
        Planned,
        /// Indicates the phase is actively being worked on.
        InProgress,
        /// Indicates the phase cannot proceed due to impediments.
        Blocked,
        /// Indicates the phase has been successfully finished.
        Completed,
        /// Indicates the phase was intentionally bypassed.
        Skipped,
    }
}

crate::define_string_enum! {
    /// Classifies the nature of a project issue.
    pub enum IssueType [strum = "snake_case", schema] {
        /// Represents a unit of work to be performed.
        Task,
        /// Represents a defect or error that needs resolution.
        Bug,
        /// Represents a new capability or functionality.
        Feature,
        /// Represents an improvement to existing functionality.
        Enhancement,
        /// Represents a documentation-only change.
        Documentation,
    }
}

crate::define_string_enum! {
    /// Tracks the lifecycle state of an issue.
    pub enum IssueStatus [strum = "snake_case", schema] {
        /// Indicates the issue is new and awaiting action.
        Open,
        /// Indicates the issue is actively being worked on.
        InProgress,
        /// Indicates the issue is blocked by external factors.
        Blocked,
        /// Indicates the issue work has been finished.
        Resolved,
        /// Indicates the issue is verified and fully completed.
        Closed,
    }
}

crate::define_string_enum! {
    /// Defines the relationship between two project issues.
    pub enum DependencyType [strum = "snake_case"] {
        /// Indicates the source issue prevents the target issue from starting.
        Blocks,
        /// Indicates the source issue is relevant to the target issue context.
        RelatesTo,
        /// Indicates the source issue describes the same problem as the target issue.
        DuplicateOf,
        /// Indicates the source issue is the parent container of the target issue.
        ParentOf,
    }
}

crate::define_entity! {
    /// Represents a distinct stage in the project roadmap.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProjectPhase { id, project_id, created_at, updated_at } {
        /// Name of the phase.
        pub name: String,
        /// Detailed description of phase objectives.
        pub description: String,
        /// Order of execution in the roadmap.
        pub sequence: i32,
        /// Current lifecycle status.
        pub status: PhaseStatus,
        /// When work on the phase began.
        pub started_at: Option<i64>,
        /// When the phase was completed.
        pub completed_at: Option<i64>,
    }
}

crate::define_entity! {
    /// Represents a unit of work, bug, or feature request within a project.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ProjectIssue { id, org_id, project_id, created_at, updated_at } {
        /// User identifier of the issue creator.
        pub created_by: String,
        /// Optional phase this issue belongs to.
        #[serde(default)]
        pub phase_id: Option<String>,
        /// Concise summary of the issue.
        pub title: String,
        /// Detailed explanation of the issue or task requirements.
        pub description: String,
        /// Classification of the issue (e.g., Task, Bug, Feature).
        pub issue_type: IssueType,
        /// Current lifecycle state of the issue.
        pub status: IssueStatus,
        /// Priority 0-4 (0=critical, 4=backlog).
        pub priority: i32,
        /// User identifier of the person assigned to this issue.
        #[serde(default)]
        pub assignee: Option<String>,
        /// Denormalized label names for fast read access (inline JSON).
        pub labels: Vec<String>,
        /// Estimated effort in minutes.
        #[serde(default)]
        pub estimated_minutes: Option<i64>,
        /// Actual effort in minutes.
        #[serde(default)]
        pub actual_minutes: Option<i64>,
        /// Free-form operational notes.
        #[serde(default)]
        pub notes: String,
        /// Free-form design notes.
        #[serde(default)]
        pub design: String,
        /// Optional parent issue identifier for sub-task relationships.
        #[serde(default)]
        pub parent_issue_id: Option<String>,
        /// Timestamp when the issue was closed (Unix epoch).
        #[serde(default)]
        pub closed_at: Option<i64>,
        /// Human-readable reason why the issue was closed.
        #[serde(default)]
        pub closed_reason: String,
    }
}

crate::define_entity! {
    /// Represents a directed relationship between two issues.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProjectDependency { id, created_at } {
        /// Source issue of the relationship.
        pub from_issue_id: String,
        /// Target issue of the relationship.
        pub to_issue_id: String,
        /// Type of relationship (e.g., `Blocks`, `RelatesTo`).
        pub dependency_type: DependencyType,
    }
}

crate::define_entity! {
    /// Represents a recorded architectural or project decision.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ProjectDecision { id, project_id, created_at } {
        /// Optional issue this decision relates to.
        pub issue_id: Option<String>,
        /// Title of the decision.
        pub title: String,
        /// Context that led to the decision.
        pub context: String,
        /// The choice made.
        pub decision: String,
        /// Known side effects or consequences.
        pub consequences: String,
    }
}

/// Filter for querying project issues with optional constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueFilter {
    /// Filter by project identifier.
    pub project_id: Option<String>,
    /// Filter by project phase identifier.
    pub phase_id: Option<String>,
    /// Filter by type of issue (e.g., bug, task).
    pub issue_type: Option<IssueType>,
    /// Filter by issue status (e.g., open, closed).
    pub status: Option<IssueStatus>,
    /// Filter by priority level.
    pub priority: Option<i32>,
    /// Filter by assigned user.
    pub assignee: Option<String>,
    /// Filter by applied label/tag.
    pub label: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}
