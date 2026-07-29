//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#core-entities)
//!
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::value_objects::RepositoryId;

/// `VcsRepository` entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsRepository {
    id: RepositoryId,
    path: PathBuf,
    default_branch: String,
    branches: Vec<String>,
    remote_url: Option<String>,
}

impl VcsRepository {
    /// Creates a new `VcsRepository`.
    #[must_use]
    pub fn new(
        id: RepositoryId,
        path: PathBuf,
        default_branch: String,
        branches: Vec<String>,
        remote_url: Option<String>,
    ) -> Self {
        Self {
            id,
            path,
            default_branch,
            branches,
            remote_url,
        }
    }

    #[must_use]
    /// Returns the repository ID.
    pub fn id(&self) -> &RepositoryId {
        &self.id
    }

    /// Returns the repository path.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the default branch.
    #[must_use]
    pub fn default_branch(&self) -> &str {
        &self.default_branch
    }

    /// Returns the list of branches.
    #[must_use]
    pub fn branches(&self) -> &[String] {
        &self.branches
    }

    /// Returns the remote URL.
    #[must_use]
    pub fn remote_url(&self) -> Option<&str> {
        self.remote_url.as_deref()
    }
}
