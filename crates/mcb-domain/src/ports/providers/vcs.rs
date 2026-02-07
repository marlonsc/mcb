//! Version Control System provider port for repository operations.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::entities::vcs::{RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository};
use crate::error::Result;

/// Version Control System provider for repository operations.
///
/// Abstraction over version control systems (Git, Mercurial, SVN, etc.).
/// The current implementation focuses on Git, but the trait is designed
/// to support other VCS implementations in the future.
#[async_trait]
pub trait VcsProvider: Send + Sync {
    /// Open a repository at the given path
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository>;

    /// Get unique repository identifier
    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId;

    /// List all local branches in repository
    async fn list_branches(&self, repo: &VcsRepository) -> Result<Vec<VcsBranch>>;

    /// Get commit history for a branch with optional limit
    async fn commit_history(
        &self,
        repo: &VcsRepository,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>>;

    /// List files in a branch at HEAD
    async fn list_files(&self, repo: &VcsRepository, branch: &str) -> Result<Vec<PathBuf>>;

    /// Read file content from a branch at HEAD
    async fn read_file(&self, repo: &VcsRepository, branch: &str, path: &Path) -> Result<String>;

    /// VCS type name (e.g., "git", "mercurial", "svn")
    fn vcs_name(&self) -> &str;

    /// Compare two refs and return the diff
    async fn diff_refs(
        &self,
        repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff>;
}
