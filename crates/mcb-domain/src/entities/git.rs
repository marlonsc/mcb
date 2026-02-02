//! Git entities for repository, branch, and commit information.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique repository identifier based on root commit hash.
///
/// The repository ID is derived from the first commit in the repository,
/// making it stable across clones and renames.
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

/// Git repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub id: RepositoryId,
    pub path: PathBuf,
    pub default_branch: String,
    pub branches: Vec<String>,
    pub remote_url: Option<String>,
}

/// Git branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub head_commit: String,
    pub is_default: bool,
    pub upstream: Option<String>,
}

/// Git commit metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub timestamp: i64,
    pub parent_hashes: Vec<String>,
}

impl GitCommit {
    #[must_use]
    pub fn short_hash(&self) -> &str {
        if self.hash.len() >= 7 {
            &self.hash[..7]
        } else {
            &self.hash
        }
    }

    #[must_use]
    pub fn first_line(&self) -> &str {
        self.message.lines().next().unwrap_or(&self.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: PathBuf,
    pub status: DiffStatus,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefDiff {
    pub base_ref: String,
    pub head_ref: String,
    pub files: Vec<FileDiff>,
    pub total_additions: usize,
    pub total_deletions: usize,
}
