//! Version control system provider ports.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::entities::vcs::{RefDiff, VcsBranch, VcsCommit, VcsRepository};
use crate::error::Result;
use crate::value_objects::RepositoryId;

/// Version Control System provider for repository operations.
#[async_trait]
pub trait VcsProvider: Send + Sync {
    /// Open an existing repository at the specified filesystem path.
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository>;

    /// Extract a unique identifier from a repository object.
    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId;

    /// List all local and remote branches for the repository.
    async fn list_branches(&self, repo: &VcsRepository) -> Result<Vec<VcsBranch>>;

    /// Retrieve the commit history for a specific branch.
    async fn commit_history(
        &self,
        repo: &VcsRepository,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>>;

    /// List all tracked files in the given branch.
    async fn list_files(&self, repo: &VcsRepository, branch: &str) -> Result<Vec<PathBuf>>;

    /// Read the full content of a file from a specific commit/branch.
    async fn read_file(&self, repo: &VcsRepository, branch: &str, path: &Path) -> Result<String>;

    /// Get the unique name of this VCS implementation (e.g., "git").
    fn vcs_name(&self) -> &str;

    /// Calculate the difference between two references (branches, tags, or SHAs).
    async fn diff_refs(
        &self,
        repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff>;

    /// Recursive search for all repositories within a root directory.
    async fn list_repositories(&self, root: &Path) -> Result<Vec<VcsRepository>>;
}
