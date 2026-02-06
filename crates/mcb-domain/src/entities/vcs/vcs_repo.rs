pub use crate::value_objects::RepositoryId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsRepository {
    id: RepositoryId,
    path: PathBuf,
    default_branch: String,
    branches: Vec<String>,
    remote_url: Option<String>,
}

impl VcsRepository {
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
    pub fn id(&self) -> &RepositoryId {
        &self.id
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn default_branch(&self) -> &str {
        &self.default_branch
    }

    pub fn branches(&self) -> &[String] {
        &self.branches
    }

    pub fn remote_url(&self) -> Option<&str> {
        self.remote_url.as_deref()
    }
}
