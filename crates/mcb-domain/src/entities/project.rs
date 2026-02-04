//! Project entities for repository management and detection.

use serde::{Deserialize, Serialize};

/// Registered project in MCB - serves as root entity linking collections, observations, and file hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Project type detected from manifest files
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Rust project with Cargo.toml
    Cargo {
        name: String,
        version: String,
        dependencies: Vec<String>,
    },
    /// Node.js project with package.json
    Npm {
        name: String,
        version: String,
        dependencies: Vec<String>,
    },
    /// Python project with pyproject.toml or requirements.txt
    Python {
        name: String,
        version: Option<String>,
        dependencies: Vec<String>,
    },
    /// Go project with go.mod
    Go {
        module: String,
        go_version: String,
        dependencies: Vec<String>,
    },
    /// Maven project with pom.xml
    Maven {
        group_id: String,
        artifact_id: String,
        version: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
    Planned,
    InProgress,
    Blocked,
    Completed,
    Skipped,
}

impl PhaseStatus {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    Task,
    Bug,
    Feature,
    Enhancement,
    Documentation,
}

impl IssueType {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueStatus {
    Open,
    InProgress,
    Blocked,
    Resolved,
    Closed,
}

impl IssueStatus {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    Blocks,
    RelatesTo,
    DuplicateOf,
    ParentOf,
}

impl DependencyType {
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

/// A phase in a project's roadmap/workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPhase {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    /// Order in the roadmap (1-indexed).
    pub sequence: i32,
    pub status: PhaseStatus,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// An issue/task within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssue {
    pub id: String,
    pub project_id: String,
    /// Optional phase this issue belongs to.
    pub phase_id: Option<String>,
    pub title: String,
    pub description: String,
    pub issue_type: IssueType,
    pub status: IssueStatus,
    /// Priority 0-4 (0=critical, 4=backlog).
    pub priority: i32,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
}

/// Dependency/relationship between two issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDependency {
    pub id: String,
    pub from_issue_id: String,
    pub to_issue_id: String,
    pub dependency_type: DependencyType,
    pub created_at: i64,
}

/// A logged decision in the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDecision {
    pub id: String,
    pub project_id: String,
    /// Optional issue this decision relates to.
    pub issue_id: Option<String>,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub created_at: i64,
}
