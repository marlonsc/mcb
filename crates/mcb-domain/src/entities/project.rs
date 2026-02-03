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
