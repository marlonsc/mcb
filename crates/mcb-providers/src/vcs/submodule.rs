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
    // TODO(qlty): Function with high complexity (count = 33): collect_submodules_sync
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
                let path = submodule.path().to_str().unwrap_or_default().to_owned();

                // Check for circular references
                let unique_key = format!("{parent_id}:{path}");
                if visited.contains(&unique_key) {
                    mcb_domain::warn!(
                        "submodule",
                        "Circular submodule reference detected, skipping",
                        &path
                    );
                    continue;
                }
                visited.insert(unique_key);

                // Get submodule URL (may be None for orphaned submodules)
                let url = match submodule.url() {
                    Some(u) => u.to_owned(),
                    None => {
                        mcb_domain::warn!(
                            "submodule",
                            "Orphaned submodule (no URL in .gitmodules), skipping",
                            &path
                        );
                        continue;
                    }
                };

                // Get commit hash (submodules are typically in detached HEAD)
                let commit_hash = submodule
                    .head_id()
                    .map(|oid| oid.to_string())
                    .unwrap_or_default();

                // Get submodule name
                let name = submodule
                    .name()
                    .map_or_else(|| path.clone(), ToString::to_string);

                // Check if submodule is initialized
                let workdir = current_repo.workdir().unwrap_or_else(|| Path::new(""));
                let submodule_path = workdir.join(&path);
                let is_initialized =
                    submodule_path.join(".git").exists() || submodule_path.join(".git").is_file(); // .git can be a file for nested

                if skip_uninitialized && !is_initialized {
                    mcb_domain::debug!("submodule", "Skipping uninitialized submodule", &path);
                    continue;
                }

                let submodule_id = format!("{parent_id}:{path}");

                let info = SubmoduleInfo {
                    id: submodule_id,
                    path: path.clone(),
                    url,
                    commit_hash,
                    parent_repo_id: parent_id.clone(),
                    depth: depth + 1,
                    name,
                    is_initialized,
                };

                results.push(info);

                // Try to open submodule for recursive processing
                if is_initialized {
                    match submodule.open() {
                        Ok(sub_repo) => {
                            let sub_id = format!("{parent_id}:{path}");
                            queue.push_back((sub_repo, sub_id, depth + 1));
                        }
                        Err(e) => {
                            mcb_domain::warn!(
                                "submodule",
                                "Cannot access submodule repository, skipping nested submodules",
                                &format!("path = {path}, error = {e}")
                            );
                            // Continue with other submodules - don't block parent
                        }
                    }
                }
            }
        }

        mcb_domain::info!(
            "submodule",
            "Submodule discovery complete",
            &format!("count = {}, max_depth = {}", results.len(), max_depth)
        );

        Ok(results)
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
