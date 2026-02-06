use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId(pub String);

impl RepositoryId {
    #[must_use]
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RepositoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RepositoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

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
