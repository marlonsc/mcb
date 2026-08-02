//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Submodule traversal service using git2.

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use git2::Repository;
use mcb_domain::entities::submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
use mcb_domain::error::{Error, Result};

/// Context for building a single [`SubmoduleInfo`] during BFS traversal.
struct BuildContext<'a> {
    current_repo: &'a Repository,
    parent_id: &'a str,
    depth: usize,
    skip_uninitialized: bool,
}

/// Provider for discovering and traversing git submodules
pub struct SubmoduleProvider {
    config: SubmoduleDiscoveryConfig,
}

impl SubmoduleProvider {
    /// Create a new `SubmoduleProvider` with given configuration
    #[must_use]
    pub fn new(config: SubmoduleDiscoveryConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(SubmoduleDiscoveryConfig::default())
    }

    /// Collect all submodules from a repository up to configured depth
    ///
    /// Uses BFS (breadth-first search) to traverse submodules level by level.
    /// Handles edge cases:
    /// - Orphaned submodules (missing .gitmodules entry) - skipped with warning
    /// - Inaccessible submodule URLs - skipped, parent indexing continues
    /// - Detached HEAD - uses `head_id()` for commit hash
    /// - Circular references - tracked via visited set
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be opened or submodule traversal fails.
    pub async fn collect_submodules(
        &self,
        repo_path: &Path,
        parent_repo_id: &str,
    ) -> Result<Vec<SubmoduleInfo>> {
        // Use spawn_blocking for git2 operations (not async-safe)
        let repo_path = repo_path.to_path_buf();
        let parent_id = parent_repo_id.to_owned();
        let max_depth = self.config.max_depth;
        let skip_uninitialized = self.config.skip_uninitialized;
        let continue_on_error = self.config.continue_on_error;

        tokio::task::spawn_blocking(move || {
            Self::collect_submodules_sync(
                &repo_path,
                &parent_id,
                max_depth,
                skip_uninitialized,
                continue_on_error,
            )
        })
        .await
        .map_err(|e| Error::internal(format!("Submodule collection task panicked: {e}")))?
    }

    /// Synchronously collects submodules using git2 (thread-blocking).
    fn collect_submodules_sync(
        repo_path: &Path,
        parent_repo_id: &str,
        max_depth: usize,
        skip_uninitialized: bool,
        continue_on_error: bool,
    ) -> Result<Vec<SubmoduleInfo>> {
        let repo = Repository::discover(repo_path).map_err(|e| {
            Error::internal(format!("Failed to open repository at {repo_path:?}: {e}"))
        })?;

        let mut results = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(Repository, String, usize)> = VecDeque::new();
        queue.push_back((repo, parent_repo_id.to_owned(), 0));

        while let Some((current_repo, parent_id, depth)) = queue.pop_front() {
            if Self::at_max_depth(depth, max_depth) {
                continue;
            }

            let Some(submodules) =
                Self::list_submodules(&current_repo, continue_on_error).transpose()?
            else {
                continue;
            };

            for submodule in submodules {
                let ctx = BuildContext {
                    current_repo: &current_repo,
                    parent_id: &parent_id,
                    depth,
                    skip_uninitialized,
                };
                let Some(info) = Self::build_submodule_info(&submodule, &ctx, &mut visited) else {
                    continue;
                };

                Self::enqueue_nested(&submodule, &info, depth, &mut queue);
                results.push(info);
            }
        }

        Self::log_discovery_complete(results.len(), max_depth);
        Ok(results)
    }

    /// Returns `true` when the current depth has reached the configured limit.
    fn at_max_depth(depth: usize, max_depth: usize) -> bool {
        if depth >= max_depth {
            mcb_domain::debug!(
                "submodule",
                "Max submodule depth reached, stopping traversal",
                &format!("depth = {depth}, max_depth = {max_depth}")
            );
            true
        } else {
            false
        }
    }

    /// Log completion of submodule discovery.
    fn log_discovery_complete(count: usize, max_depth: usize) {
        mcb_domain::info!(
            "submodule",
            "Submodule discovery complete",
            &format!("count = {count}, max_depth = {max_depth}")
        );
    }

    /// List submodules of a repository, mapping the BFS error policy.
    ///
    /// Returns `None` when listing failed and `continue_on_error` is set (skip this node),
    /// `Some(Ok(..))` on success, and `Some(Err(..))` when the failure must abort traversal.
    fn list_submodules(
        repo: &Repository,
        continue_on_error: bool,
    ) -> Option<Result<Vec<git2::Submodule<'_>>>> {
        match repo.submodules() {
            Ok(s) => Some(Ok(s)),
            Err(e) => {
                mcb_domain::warn!("submodule", "Failed to list submodules", &e.to_string());
                if continue_on_error {
                    None
                } else {
                    Some(Err(Error::internal(format!(
                        "Failed to list submodules: {e}"
                    ))))
                }
            }
        }
    }

    /// Open an initialized submodule and enqueue it for nested traversal.
    fn enqueue_nested(
        submodule: &git2::Submodule<'_>,
        info: &SubmoduleInfo,
        depth: usize,
        queue: &mut VecDeque<(Repository, String, usize)>,
    ) {
        if !info.is_initialized {
            return;
        }
        match submodule.open() {
            Ok(sub_repo) => queue.push_back((sub_repo, info.id.clone(), depth + 1)),
            Err(e) => mcb_domain::warn!(
                "submodule",
                "Cannot access submodule repository, skipping nested submodules",
                &format!("path = {}, error = {e}", info.path)
            ),
        }
    }

    /// Build a `SubmoduleInfo` for one submodule, or `None` when it must be skipped
    /// (circular reference, orphaned, or uninitialized when skipping is enabled).
    fn build_submodule_info(
        submodule: &git2::Submodule<'_>,
        ctx: &BuildContext<'_>,
        visited: &mut HashSet<String>,
    ) -> Option<SubmoduleInfo> {
        let path = submodule.path().to_str().unwrap_or_default().to_owned();

        if !visited.insert(format!("{}:{path}", ctx.parent_id)) {
            mcb_domain::warn!(
                "submodule",
                "Circular submodule reference detected, skipping",
                &path
            );
            return None;
        }

        let Some(url) = submodule.url().map(str::to_owned) else {
            mcb_domain::warn!(
                "submodule",
                "Orphaned submodule (no URL in .gitmodules), skipping",
                &path
            );
            return None;
        };

        let is_initialized = Self::is_submodule_initialized(ctx.current_repo, &path);
        if ctx.skip_uninitialized && !is_initialized {
            mcb_domain::debug!("submodule", "Skipping uninitialized submodule", &path);
            return None;
        }

        let commit_hash = submodule
            .head_id()
            .map(|oid| oid.to_string())
            .unwrap_or_default();
        let name = submodule.name().unwrap_or(&path).to_owned();

        Some(SubmoduleInfo {
            id: format!("{}:{path}", ctx.parent_id),
            path,
            url,
            commit_hash,
            parent_repo_id: ctx.parent_id.to_owned(),
            depth: ctx.depth + 1,
            name,
            is_initialized,
        })
    }

    /// Whether a submodule has a populated working tree (initialized).
    ///
    /// `.git` can be a file (gitlink) for nested submodules.
    fn is_submodule_initialized(current_repo: &Repository, path: &str) -> bool {
        let workdir = current_repo.workdir().unwrap_or_else(|| Path::new(""));
        let git_entry = workdir.join(path).join(".git");
        git_entry.exists() || git_entry.is_file()
    }
}

/// Convenience function for collecting submodules with default config
///
/// # Errors
///
/// Returns an error if submodule discovery fails.
pub async fn collect_submodules(
    repo_path: &Path,
    parent_repo_id: &str,
) -> Result<Vec<SubmoduleInfo>> {
    SubmoduleProvider::with_defaults()
        .collect_submodules(repo_path, parent_repo_id)
        .await
}

/// Convenience function with custom depth
///
/// # Errors
///
/// Returns an error if submodule discovery fails.
pub async fn collect_submodules_with_depth(
    repo_path: &Path,
    parent_repo_id: &str,
    max_depth: usize,
) -> Result<Vec<SubmoduleInfo>> {
    let config = SubmoduleDiscoveryConfig {
        max_depth,
        ..Default::default()
    };
    SubmoduleProvider::new(config)
        .collect_submodules(repo_path, parent_repo_id)
        .await
}
