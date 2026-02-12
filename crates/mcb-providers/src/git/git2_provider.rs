//! Git2-based implementation of VcsProvider

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git2::{BranchType, Repository, Sort};
use mcb_domain::{
    entities::vcs::{
        DiffStatus, FileDiff, RefDiff, RepositoryId, VcsBranch, VcsCommit, VcsRepository,
    },
    error::{Error, Result},
    ports::providers::VcsProvider,
};
use uuid::Uuid;

/// Git implementation of VcsProvider using libgit2.
///
/// Constructed by [`mcb_infrastructure::di::vcs`] module for DI registration.
#[allow(dead_code)] // Cross-crate usage: constructed in mcb-infrastructure::di::vcs
pub struct Git2Provider;

impl Git2Provider {
    /// Create a new Git2 provider instance
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn open_repo(path: &Path) -> Result<Repository> {
        Repository::open(path).map_err(|e| {
            if e.code() == git2::ErrorCode::NotFound {
                Error::repository_not_found(path.display().to_string())
            } else {
                Error::vcs_with_source(format!("Failed to open repository: {}", path.display()), e)
            }
        })
    }

    fn get_root_commit_hash(repo: &Repository) -> Result<String> {
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| Error::vcs_with_source("Failed to create revwalk", e))?;

        revwalk
            .push_head()
            .map_err(|e| Error::vcs_with_source("Failed to push HEAD to revwalk", e))?;

        revwalk.set_sorting(Sort::TIME | Sort::REVERSE).ok();

        let first_oid = revwalk
            .next()
            .ok_or_else(|| Error::vcs("Repository has no commits"))?
            .map_err(|e| Error::vcs_with_source("Failed to get first commit", e))?;

        Ok(first_oid.to_string())
    }

    fn get_default_branch(repo: &Repository) -> String {
        repo.head()
            .ok()
            .and_then(|head| head.shorthand().map(String::from))
            .unwrap_or_else(|| "main".to_string())
    }

    fn get_remote_url(repo: &Repository) -> Option<String> {
        repo.find_remote("origin")
            .ok()
            .and_then(|remote| remote.url().map(String::from))
    }

    fn list_branch_names(repo: &Repository) -> Result<Vec<String>> {
        let branches = repo
            .branches(Some(BranchType::Local))
            .map_err(|e| Error::vcs_with_source("Failed to list branches", e))?;

        let mut names = Vec::new();
        for branch_result in branches {
            let (branch, _) =
                branch_result.map_err(|e| Error::vcs_with_source("Failed to read branch", e))?;
            if let Some(name) = branch.name().ok().flatten() {
                names.push(name.to_string());
            }
        }

        Ok(names)
    }
}

impl Default for Git2Provider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VcsProvider for Git2Provider {
    async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
        let repo = Self::open_repo(path)?;

        let id = RepositoryId::new(Self::get_root_commit_hash(&repo)?);
        let default_branch = Self::get_default_branch(&repo);
        let branches = Self::list_branch_names(&repo)?;
        let remote_url = Self::get_remote_url(&repo);

        Ok(VcsRepository::new(
            id,
            path.to_path_buf(),
            default_branch,
            branches,
            remote_url,
        ))
    }

    fn repository_id(&self, repo: &VcsRepository) -> RepositoryId {
        repo.id().clone()
    }

    async fn list_branches(&self, repo: &VcsRepository) -> Result<Vec<VcsBranch>> {
        let git_repo = Self::open_repo(repo.path())?;

        let branches = git_repo
            .branches(Some(BranchType::Local))
            .map_err(|e| Error::vcs_with_source("Failed to list branches", e))?;

        let mut result = Vec::new();
        for branch_result in branches {
            let (branch, _) =
                branch_result.map_err(|e| Error::vcs_with_source("Failed to read branch", e))?;

            let name = branch
                .name()
                .ok()
                .flatten()
                .map(String::from)
                .unwrap_or_default();

            let head_commit = branch
                .get()
                .peel_to_commit()
                .map(|c| c.id().to_string())
                .unwrap_or_default();

            let is_default = name == repo.default_branch();

            let upstream = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(String::from));

            result.push(VcsBranch::new(
                format!("{}::{}", repo.id(), name),
                name,
                head_commit,
                is_default,
                upstream,
            ));
        }

        Ok(result)
    }

    async fn commit_history(
        &self,
        repo: &VcsRepository,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VcsCommit>> {
        let git_repo = Self::open_repo(repo.path())?;

        let branch_ref = git_repo
            .find_branch(branch, BranchType::Local)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::NotFound {
                    Error::branch_not_found(branch)
                } else {
                    Error::vcs_with_source(format!("Failed to find branch: {branch}"), e)
                }
            })?;

        let branch_commit = branch_ref
            .get()
            .peel_to_commit()
            .map_err(|e| Error::vcs_with_source("Failed to get branch commit", e))?;

        let mut revwalk = git_repo
            .revwalk()
            .map_err(|e| Error::vcs_with_source("Failed to create revwalk", e))?;

        revwalk
            .push(branch_commit.id())
            .map_err(|e| Error::vcs_with_source("Failed to push commit to revwalk", e))?;

        revwalk.set_sorting(Sort::TIME).ok();

        let mut commits = Vec::new();
        let max_commits = limit.unwrap_or(usize::MAX);

        for oid_result in revwalk {
            if commits.len() >= max_commits {
                break;
            }

            let oid =
                oid_result.map_err(|e| Error::vcs_with_source("Failed to iterate commits", e))?;

            let commit = git_repo
                .find_commit(oid)
                .map_err(|e| Error::vcs_with_source("Failed to find commit", e))?;

            let author = commit.author();
            let parent_hashes: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();

            commits.push(VcsCommit::new(
                format!("{}:{}", repo.id(), oid),
                oid.to_string(),
                commit.message().unwrap_or("").to_string(),
                author.name().unwrap_or("Unknown").to_string(),
                author.email().unwrap_or("").to_string(),
                commit.time().seconds(),
                parent_hashes,
            ));
        }

        Ok(commits)
    }

    async fn list_files(&self, repo: &VcsRepository, branch: &str) -> Result<Vec<PathBuf>> {
        let git_repo = Self::open_repo(repo.path())?;

        let branch_ref = git_repo
            .find_branch(branch, BranchType::Local)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::NotFound {
                    Error::branch_not_found(branch)
                } else {
                    Error::vcs_with_source(format!("Failed to find branch: {branch}"), e)
                }
            })?;

        let tree = branch_ref
            .get()
            .peel_to_tree()
            .map_err(|e| Error::vcs_with_source("Failed to get branch tree", e))?;

        let mut files = Vec::new();
        tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob)
                && let Some(name) = entry.name()
            {
                let path = if dir.is_empty() {
                    PathBuf::from(name)
                } else {
                    PathBuf::from(dir).join(name)
                };
                files.push(path);
            }
            git2::TreeWalkResult::Ok
        })
        .map_err(|e| Error::vcs_with_source("Failed to walk tree", e))?;

        Ok(files)
    }

    async fn read_file(&self, repo: &VcsRepository, branch: &str, path: &Path) -> Result<String> {
        let git_repo = Self::open_repo(repo.path())?;

        let branch_ref = git_repo
            .find_branch(branch, BranchType::Local)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::NotFound {
                    Error::branch_not_found(branch)
                } else {
                    Error::vcs_with_source(format!("Failed to find branch: {branch}"), e)
                }
            })?;

        let tree = branch_ref
            .get()
            .peel_to_tree()
            .map_err(|e| Error::vcs_with_source("Failed to get branch tree", e))?;

        let path_str = path.to_string_lossy();
        let entry = tree.get_path(path).map_err(|e| {
            Error::vcs_with_source(format!("File not found in branch: {path_str}"), e)
        })?;

        let blob = entry
            .to_object(&git_repo)
            .and_then(|obj| obj.peel_to_blob());

        match blob {
            Ok(blob) => {
                if blob.is_binary() {
                    return Err(Error::vcs(format!("Binary file: {path_str}")));
                }
                String::from_utf8(blob.content().to_vec())
                    .map_err(|e| Error::vcs_with_source(format!("Invalid UTF-8: {path_str}"), e))
            }
            Err(e) => Err(Error::vcs_with_source(
                format!("Failed to read file: {path_str}"),
                e,
            )),
        }
    }

    fn vcs_name(&self) -> &str {
        "git"
    }

    async fn diff_refs(
        &self,
        repo: &VcsRepository,
        base_ref: &str,
        head_ref: &str,
    ) -> Result<RefDiff> {
        let git_repo = Self::open_repo(repo.path())?;

        let base_obj = git_repo
            .revparse_single(base_ref)
            .map_err(|e| Error::vcs_with_source(format!("Failed to resolve ref: {base_ref}"), e))?;
        let head_obj = git_repo
            .revparse_single(head_ref)
            .map_err(|e| Error::vcs_with_source(format!("Failed to resolve ref: {head_ref}"), e))?;

        let base_tree = base_obj
            .peel_to_tree()
            .map_err(|e| Error::vcs_with_source("Failed to get base tree", e))?;
        let head_tree = head_obj
            .peel_to_tree()
            .map_err(|e| Error::vcs_with_source("Failed to get head tree", e))?;

        let diff = git_repo
            .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)
            .map_err(|e| Error::vcs_with_source("Failed to create diff", e))?;

        let mut files = Vec::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;

        diff.foreach(
            &mut |delta, _| {
                let status = match delta.status() {
                    git2::Delta::Added => DiffStatus::Added,
                    git2::Delta::Deleted => DiffStatus::Deleted,
                    git2::Delta::Modified => DiffStatus::Modified,
                    git2::Delta::Renamed => DiffStatus::Renamed,
                    _ => DiffStatus::Modified,
                };

                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(PathBuf::from)
                    .unwrap_or_default();

                files.push(FileDiff {
                    id: Uuid::new_v4().to_string(),
                    path,
                    status,
                    additions: 0,
                    deletions: 0,
                });
                true
            },
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                let origin = line.origin();
                if origin == '+' {
                    total_additions += 1;
                } else if origin == '-' {
                    total_deletions += 1;
                }
                true
            }),
        )
        .map_err(|e| Error::vcs_with_source("Failed to iterate diff", e))?;

        Ok(RefDiff {
            id: Uuid::new_v4().to_string(),
            base_ref: base_ref.to_string(),
            head_ref: head_ref.to_string(),
            files,
            total_additions,
            total_deletions,
        })
    }

    async fn list_repositories(&self, root: &Path) -> Result<Vec<VcsRepository>> {
        use walkdir::WalkDir;

        let mut repositories = Vec::new();

        // Check if root itself is a repository
        let root_is_repo = root.join(".git").exists();
        if root_is_repo && let Ok(repo) = self.open_repository(root).await {
            repositories.push(repo);
        }

        // Recursively scan subdirectories for repositories (limit depth to 3)
        // Use walkdir instead of fs::read_dir for recursion
        let walker = WalkDir::new(root)
            .min_depth(1)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_dir());

        for entry in walker {
            let path = entry.path();
            // Skip .git directories themselves
            if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }

            // Check for .git child directory
            if path.join(".git").exists()
                && let Ok(repo) = self.open_repository(path).await
            {
                repositories.push(repo);
            }
        }

        Ok(repositories)
    }
}
