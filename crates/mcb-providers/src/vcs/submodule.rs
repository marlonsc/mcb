//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Submodule traversal service using git2.

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use git2::Repository;
use mcb_domain::entities::submodule::{SubmoduleDiscoveryConfig, SubmoduleInfo};
use mcb_domain::error::{Error, Result};

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

        // BFS queue: (Repository, parent_id, current_depth)
        let mut queue: VecDeque<(Repository, String, usize)> = VecDeque::new();
        queue.push_back((repo, parent_repo_id.to_owned(), 0));

        while let Some((current_repo, parent_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                mcb_domain::debug!(
                    "submodule",
                    "Max submodule depth reached, stopping traversal",
                    &format!("depth = {depth}, max_depth = {max_depth}")
                );
                continue;
            }

            let submodules = match current_repo.submodules() {
                Ok(s) => s,
                Err(e) => {
                    mcb_domain::warn!("submodule", "Failed to list submodules", &e.to_string());
                    if continue_on_error {
                        continue;
                    }
                    return Err(Error::internal(format!("Failed to list submodules: {e}")));
                }
            };

            for submodule in submodules {
                let Some(info) = Self::build_submodule_info(
                    &submodule,
                    &current_repo,
                    &parent_id,
                    depth,
                    skip_uninitialized,
                    &mut visited,
                ) else {
                    continue;
                };

                // Try to open submodule for recursive processing before moving `info`.
                if info.is_initialized {
                    match submodule.open() {
                        Ok(sub_repo) => {
                            queue.push_back((sub_repo, info.id.clone(), depth + 1));
                        }
                        Err(e) => {
                            mcb_domain::warn!(
                                "submodule",
                                "Cannot access submodule repository, skipping nested submodules",
                                &format!("path = {}, error = {e}", info.path)
                            );
                        }
                    }
                }

                results.push(info);
            }
        }

        mcb_domain::info!(
            "submodule",
            "Submodule discovery complete",
            &format!("count = {}, max_depth = {}", results.len(), max_depth)
        );

        Ok(results)
    }

    /// Build a `SubmoduleInfo` for one submodule, or `None` when it must be skipped
    /// (circular reference, orphaned, or uninitialized when skipping is enabled).
    fn build_submodule_info(
        submodule: &git2::Submodule<'_>,
        current_repo: &Repository,
        parent_id: &str,
        depth: usize,
        skip_uninitialized: bool,
        visited: &mut HashSet<String>,
    ) -> Option<SubmoduleInfo> {
        let path = submodule.path().to_str().unwrap_or_default().to_owned();

        let unique_key = format!("{parent_id}:{path}");
        if !visited.insert(unique_key) {
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

        let commit_hash = submodule
            .head_id()
            .map(|oid| oid.to_string())
            .unwrap_or_default();
        let name = submodule.name().unwrap_or(&path).to_owned();

        let workdir = current_repo.workdir().unwrap_or_else(|| Path::new(""));
        let submodule_path = workdir.join(&path);
        // `.git` can be a file (gitlink) for nested submodules.
        let is_initialized =
            submodule_path.join(".git").exists() || submodule_path.join(".git").is_file();

        if skip_uninitialized && !is_initialized {
            mcb_domain::debug!("submodule", "Skipping uninitialized submodule", &path);
            return None;
        }

        Some(SubmoduleInfo {
            id: format!("{parent_id}:{path}"),
            path,
            url,
            commit_hash,
            parent_repo_id: parent_id.to_owned(),
            depth: depth + 1,
            name,
            is_initialized,
        })
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
