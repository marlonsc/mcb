//! Git2-based implementation of VcsProvider

use async_trait::async_trait;
use git2::{BranchType, Repository, Sort};
use mcb_domain::{
    entities::git::{GitBranch, GitCommit, GitRepository, RepositoryId},
    error::{Error, Result},
    ports::providers::VcsProvider,
};
use std::path::{Path, PathBuf};

/// Git implementation of VcsProvider using libgit2.
pub struct Git2Provider;

impl Git2Provider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn open_repo(path: &Path) -> Result<Repository> {
        Repository::open(path).map_err(|e| {
            if e.code() == git2::ErrorCode::NotFound {
                Error::repository_not_found(path.display().to_string())
            } else {
                Error::git_with_source(format!("Failed to open repository: {}", path.display()), e)
            }
        })
    }

    fn get_root_commit_hash(repo: &Repository) -> Result<String> {
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| Error::git_with_source("Failed to create revwalk", e))?;

        revwalk
            .push_head()
            .map_err(|e| Error::git_with_source("Failed to push HEAD to revwalk", e))?;

        revwalk.set_sorting(Sort::TIME | Sort::REVERSE).ok();

        let first_oid = revwalk
            .next()
            .ok_or_else(|| Error::git("Repository has no commits"))?
            .map_err(|e| Error::git_with_source("Failed to get first commit", e))?;

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
            .map_err(|e| Error::git_with_source("Failed to list branches", e))?;

        let mut names = Vec::new();
        for branch_result in branches {
            let (branch, _) =
                branch_result.map_err(|e| Error::git_with_source("Failed to read branch", e))?;
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
    async fn open_repository(&self, path: &Path) -> Result<GitRepository> {
        let repo = Self::open_repo(path)?;

        let id = RepositoryId::new(Self::get_root_commit_hash(&repo)?);
        let default_branch = Self::get_default_branch(&repo);
        let branches = Self::list_branch_names(&repo)?;
        let remote_url = Self::get_remote_url(&repo);

        Ok(GitRepository {
            id,
            path: path.to_path_buf(),
            default_branch,
            branches,
            remote_url,
        })
    }

    fn repository_id(&self, repo: &GitRepository) -> RepositoryId {
        repo.id.clone()
    }

    async fn list_branches(&self, repo: &GitRepository) -> Result<Vec<GitBranch>> {
        let git_repo = Self::open_repo(&repo.path)?;

        let branches = git_repo
            .branches(Some(BranchType::Local))
            .map_err(|e| Error::git_with_source("Failed to list branches", e))?;

        let mut result = Vec::new();
        for branch_result in branches {
            let (branch, _) =
                branch_result.map_err(|e| Error::git_with_source("Failed to read branch", e))?;

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

            let is_default = name == repo.default_branch;

            let upstream = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(String::from));

            result.push(GitBranch {
                name,
                head_commit,
                is_default,
                upstream,
            });
        }

        Ok(result)
    }

    async fn commit_history(
        &self,
        repo: &GitRepository,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<GitCommit>> {
        let git_repo = Self::open_repo(&repo.path)?;

        let branch_ref = git_repo
            .find_branch(branch, BranchType::Local)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::NotFound {
                    Error::branch_not_found(branch)
                } else {
                    Error::git_with_source(format!("Failed to find branch: {branch}"), e)
                }
            })?;

        let branch_commit = branch_ref
            .get()
            .peel_to_commit()
            .map_err(|e| Error::git_with_source("Failed to get branch commit", e))?;

        let mut revwalk = git_repo
            .revwalk()
            .map_err(|e| Error::git_with_source("Failed to create revwalk", e))?;

        revwalk
            .push(branch_commit.id())
            .map_err(|e| Error::git_with_source("Failed to push commit to revwalk", e))?;

        revwalk.set_sorting(Sort::TIME).ok();

        let mut commits = Vec::new();
        let max_commits = limit.unwrap_or(usize::MAX);

        for oid_result in revwalk {
            if commits.len() >= max_commits {
                break;
            }

            let oid =
                oid_result.map_err(|e| Error::git_with_source("Failed to iterate commits", e))?;

            let commit = git_repo
                .find_commit(oid)
                .map_err(|e| Error::git_with_source("Failed to find commit", e))?;

            let author = commit.author();
            let parent_hashes: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();

            commits.push(GitCommit {
                hash: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: author.name().unwrap_or("Unknown").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
                parent_hashes,
            });
        }

        Ok(commits)
    }

    async fn list_files(&self, repo: &GitRepository, branch: &str) -> Result<Vec<PathBuf>> {
        let git_repo = Self::open_repo(&repo.path)?;

        let branch_ref = git_repo
            .find_branch(branch, BranchType::Local)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::NotFound {
                    Error::branch_not_found(branch)
                } else {
                    Error::git_with_source(format!("Failed to find branch: {branch}"), e)
                }
            })?;

        let tree = branch_ref
            .get()
            .peel_to_tree()
            .map_err(|e| Error::git_with_source("Failed to get branch tree", e))?;

        let mut files = Vec::new();
        tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                if let Some(name) = entry.name() {
                    let path = if dir.is_empty() {
                        PathBuf::from(name)
                    } else {
                        PathBuf::from(dir).join(name)
                    };
                    files.push(path);
                }
            }
            git2::TreeWalkResult::Ok
        })
        .map_err(|e| Error::git_with_source("Failed to walk tree", e))?;

        Ok(files)
    }

    async fn read_file(&self, repo: &GitRepository, branch: &str, path: &Path) -> Result<String> {
        let git_repo = Self::open_repo(&repo.path)?;

        let branch_ref = git_repo
            .find_branch(branch, BranchType::Local)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::NotFound {
                    Error::branch_not_found(branch)
                } else {
                    Error::git_with_source(format!("Failed to find branch: {branch}"), e)
                }
            })?;

        let tree = branch_ref
            .get()
            .peel_to_tree()
            .map_err(|e| Error::git_with_source("Failed to get branch tree", e))?;

        let path_str = path.to_string_lossy();
        let entry = tree.get_path(path).map_err(|e| {
            Error::git_with_source(format!("File not found in branch: {path_str}"), e)
        })?;

        let blob = entry
            .to_object(&git_repo)
            .and_then(|obj| obj.peel_to_blob());

        match blob {
            Ok(blob) => {
                if blob.is_binary() {
                    return Err(Error::git(format!("Binary file: {path_str}")));
                }
                String::from_utf8(blob.content().to_vec())
                    .map_err(|e| Error::git_with_source(format!("Invalid UTF-8: {path_str}"), e))
            }
            Err(e) => Err(Error::git_with_source(
                format!("Failed to read file: {path_str}"),
                e,
            )),
        }
    }

    fn vcs_name(&self) -> &str {
        "git"
    }
}

#[cfg(test)]
mod tests;
