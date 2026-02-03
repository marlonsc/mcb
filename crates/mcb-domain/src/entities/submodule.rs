//! Submodule entity representing a VCS submodule within a repository.

use serde::{Deserialize, Serialize};

/// Information about a VCS submodule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleInfo {
    /// Stable identifier for the submodule (parent repo + path)
    pub id: String,
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
