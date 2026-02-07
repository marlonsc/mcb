//! Project entities for repository management and detection.

use serde::{Deserialize, Serialize};

/// Registered project in MCB - serves as root entity linking collections, observations, and file hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier for the project.
    pub id: String,
    /// Display name of the project.
    pub name: String,
    /// Absolute filesystem path to the project root.
    pub path: String,
    /// Timestamp when the project was registered (Unix epoch).
    pub created_at: i64,
    /// Timestamp when the project was last updated (Unix epoch).
    pub updated_at: i64,
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
// Phase 5: Workflow State (ADR-032)
// ============================================================================

/// Represents the execution state of a project phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
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

impl PhaseStatus {
    /// Returns the string representation of the phase status.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
        }
    }
}

impl std::str::FromStr for PhaseStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "planned" => Ok(Self::Planned),
            "in_progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "completed" => Ok(Self::Completed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(format!("Unknown phase status: {s}")),
        }
    }
}

/// Classifies the nature of a project issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
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

impl IssueType {
    /// Returns the string representation of the issue type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Enhancement => "enhancement",
            Self::Documentation => "documentation",
        }
    }
}

impl std::str::FromStr for IssueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "task" => Ok(Self::Task),
            "bug" => Ok(Self::Bug),
            "feature" => Ok(Self::Feature),
            "enhancement" => Ok(Self::Enhancement),
            "documentation" => Ok(Self::Documentation),
            _ => Err(format!("Unknown issue type: {s}")),
        }
    }
}

/// Tracks the lifecycle state of an issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueStatus {
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

impl IssueStatus {
    /// Returns the string representation of the issue status.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}

impl std::str::FromStr for IssueStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "in_progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "resolved" => Ok(Self::Resolved),
            "closed" => Ok(Self::Closed),
            _ => Err(format!("Unknown issue status: {s}")),
        }
    }
}

/// Defines the relationship between two project issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Indicates the source issue prevents the target issue from starting.
    Blocks,
    /// Indicates the source issue is relevant to the target issue context.
    RelatesTo,
    /// Indicates the source issue describes the same problem as the target issue.
    DuplicateOf,
    /// Indicates the source issue is the parent container of the target issue.
    ParentOf,
}

impl DependencyType {
    /// Returns the string representation of the dependency type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blocks => "blocks",
            Self::RelatesTo => "relates_to",
            Self::DuplicateOf => "duplicate_of",
            Self::ParentOf => "parent_of",
        }
    }
}

impl std::str::FromStr for DependencyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blocks" => Ok(Self::Blocks),
            "relates_to" => Ok(Self::RelatesTo),
            "duplicate_of" => Ok(Self::DuplicateOf),
            "parent_of" => Ok(Self::ParentOf),
            _ => Err(format!("Unknown dependency type: {s}")),
        }
    }
}

/// Represents a distinct stage in the project roadmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPhase {
    /// Unique identifier for the phase.
    pub id: String,
    /// Identifier of the project this phase belongs to.
    pub project_id: String,
    /// Display name of the phase.
    pub name: String,
    /// Detailed description of the phase's goals and scope.
    pub description: String,
    /// Order in the roadmap (1-indexed).
    pub sequence: i32,
    /// Current execution status of the phase.
    pub status: PhaseStatus,
    /// Timestamp when the phase started execution (Unix epoch).
    pub started_at: Option<i64>,
    /// Timestamp when the phase was completed (Unix epoch).
    pub completed_at: Option<i64>,
    /// Timestamp when the phase was created (Unix epoch).
    pub created_at: i64,
    /// Timestamp when the phase was last updated (Unix epoch).
    pub updated_at: i64,
}

/// Represents a unit of work, bug, or feature request within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssue {
    /// Unique identifier for the issue.
    pub id: String,
    /// Identifier of the project this issue belongs to.
    pub project_id: String,
    /// Optional phase this issue belongs to.
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
    pub assignee: Option<String>,
    /// Set of tags or categories associated with the issue.
    pub labels: Vec<String>,
    /// Timestamp when the issue was created (Unix epoch).
    pub created_at: i64,
    /// Timestamp when the issue was last updated (Unix epoch).
    pub updated_at: i64,
    /// Timestamp when the issue was closed (Unix epoch).
    pub closed_at: Option<i64>,
}

/// Represents a directed relationship between two issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDependency {
    /// Unique identifier for the dependency record.
    pub id: String,
    /// Identifier of the source issue.
    pub from_issue_id: String,
    /// Identifier of the target issue.
    pub to_issue_id: String,
    /// The nature of the relationship between the issues.
    pub dependency_type: DependencyType,
    /// Timestamp when the dependency was created (Unix epoch).
    pub created_at: i64,
}

/// Represents a recorded architectural or project decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDecision {
    /// Unique identifier for the decision.
    pub id: String,
    /// Identifier of the project this decision applies to.
    pub project_id: String,
    /// Optional issue this decision relates to.
    pub issue_id: Option<String>,
    /// Concise summary of the decision made.
    pub title: String,
    /// Background information and rationale leading to the decision.
    pub context: String,
    /// The chosen course of action or conclusion.
    pub decision: String,
    /// Expected outcomes, impacts, or side effects of the decision.
    pub consequences: String,
    /// Timestamp when the decision was recorded (Unix epoch).
    pub created_at: i64,
}
