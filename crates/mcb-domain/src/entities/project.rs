//! Project entity representing a detected project within a repository.

use serde::{Deserialize, Serialize};

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

impl ProjectType {
    /// Get the project name
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            ProjectType::Cargo { name, .. } => name,
            ProjectType::Npm { name, .. } => name,
            ProjectType::Python { name, .. } => name,
            ProjectType::Go { module, .. } => module,
            ProjectType::Maven { artifact_id, .. } => artifact_id,
        }
    }

    /// Get the project type as a string identifier
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            ProjectType::Cargo { .. } => "cargo",
            ProjectType::Npm { .. } => "npm",
            ProjectType::Python { .. } => "python",
            ProjectType::Go { .. } => "go",
            ProjectType::Maven { .. } => "maven",
        }
    }
}

/// Detected project with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProject {
    /// Path relative to repository root
    pub path: String,
    /// Detected project type with metadata
    pub project_type: ProjectType,
    /// Parent repository ID (for submodule linking)
    pub parent_repo_id: Option<String>,
}
