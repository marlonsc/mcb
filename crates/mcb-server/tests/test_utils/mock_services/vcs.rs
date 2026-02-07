//! Mock VCS Provider implementation

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use mcb_domain::entities::vcs::{
    DiffStatus, FileDiff, RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::VcsProvider;

/// Mock VCS provider for testing
pub struct MockVcsProvider {
    pub should_fail: AtomicBool,
}

impl MockVcsProvider {
    pub fn new() -> Self {
        Self {
            should_fail: AtomicBool::new(false),
        }
    }

    pub fn with_failure(self) -> Self {
        self.should_fail.store(true, Ordering::SeqCst);
        self
    }
}

impl Default for MockVcsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VcsProvider for MockVcsProvider {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(mcb_domain::error::Error::vcs("Mock failure"));
        }
        Ok(VcsRepository::new(
            RepositoryId::new("mock-repo-id".to_string()),
            path.to_path_buf(),
            "main".to_string(),
            vec!["main".to_string()],
            None,
        ))
    }

    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId {
        repo.id().clone()
    }

    async fn list_branches(&self, _repo: &VcsRepository) -> Result<Vec<VcsBranch>> {
        Ok(vec![VcsBranch::new(
            uuid::Uuid::new_v4().to_string(),
            "main".to_string(),
            "abc123".to_string(),
            true,
            None,
        )])
    }

    async fn commit_history(
        &self,
        _repo: &VcsRepository,
        _branch: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>> {
        Ok(vec![VcsCommit::new(
            uuid::Uuid::new_v4().to_string(),
            "abc123".to_string(),
            "Initial commit".to_string(),
            "Test".to_string(),
            "test@test.com".to_string(),
            0,
            vec![],
        )])
    }

    async fn list_files(&self, _repo: &VcsRepository, _branch: &str) -> Result<Vec<PathBuf>> {
        Ok(vec![PathBuf::from("README.md")])
    }

    async fn read_file(
        &self,
        _repo: &VcsRepository,
        _branch: &str,
        _path: &Path,
    ) -> Result<String> {
        Ok("# Mock File".to_string())
    }

    fn vcs_name(&self) -> &str {
        "mock"
    }

    async fn diff_refs(
        &self,
        _repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff> {
        Ok(RefDiff {
            id: "mock-ref-diff-id".to_string(),
            base_ref: base_ref.to_string(),
            head_ref: head_ref.to_string(),
            files: vec![FileDiff {
                id: "mock-file-diff-id".to_string(),
                path: PathBuf::from("README.md"),
                status: DiffStatus::Modified,
                additions: 5,
                deletions: 2,
            }],
            total_additions: 5,
            total_deletions: 2,
        })
    }
}
