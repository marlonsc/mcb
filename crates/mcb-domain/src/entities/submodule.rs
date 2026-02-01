//! Submodule entity representing a git submodule within a repository.

use serde::{Deserialize, Serialize};

/// Information about a git submodule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleInfo {
    /// Path relative to parent repository root
    pub path: String,
    /// Submodule URL (from .gitmodules)
    pub url: String,
    /// Commit hash the submodule points to
    pub commit_hash: String,
    /// Parent repository ID (for linking)
    pub parent_repo_id: String,
    /// Depth level (1 = direct submodule, 2 = nested, etc.)
    pub depth: usize,
    /// Name of the submodule (from .gitmodules)
    pub name: String,
    /// Whether the submodule is initialized
    pub is_initialized: bool,
}

impl SubmoduleInfo {
    /// Generate a collection name for this submodule
    /// Format: parent_collection/submodule_path
    #[must_use]
    pub fn collection_name(&self, parent_collection: &str) -> String {
        format!("{}/{}", parent_collection, self.path.replace('/', "-"))
    }

    /// Generate a unique repository ID for this submodule
    /// Based on parent + path for stable identification
    #[must_use]
    pub fn repo_id(&self) -> String {
        format!("{}:{}", self.parent_repo_id, self.path)
    }
}

/// Configuration for submodule discovery
#[derive(Debug, Clone)]
pub struct SubmoduleDiscoveryConfig {
    /// Maximum depth to traverse (default: 2)
    pub max_depth: usize,
    /// Whether to skip uninitialized submodules
    pub skip_uninitialized: bool,
    /// Whether to continue on errors (default: true)
    pub continue_on_error: bool,
}

impl Default for SubmoduleDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            skip_uninitialized: false,
            continue_on_error: true,
        }
    }
}
