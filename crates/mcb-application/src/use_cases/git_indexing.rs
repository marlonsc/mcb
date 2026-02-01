//! Git-aware indexing service with submodule and project detection.
//!
//! Orchestrates the indexing of git repositories including:
//! - Submodule discovery and recursive indexing
//! - Project type detection (Cargo, npm, Python, Go, Maven)
//! - Incremental indexing via file hash comparison
//! - Hierarchical collection organization

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use mcb_domain::entities::project::{DetectedProject, ProjectType};
use mcb_domain::entities::submodule::SubmoduleInfo;
use mcb_domain::error::Result;

/// Configuration for git indexing
///
/// NOTE: Submodules are ALWAYS indexed when present (user decision: automatic detection).
/// Users control depth via `submodule_depth`, not via opt-in/opt-out flag.
#[derive(Debug, Clone)]
pub struct GitIndexingOptions {
    /// Maximum submodule depth (default: 2)
    /// Set to 0 to skip submodule indexing entirely
    pub submodule_depth: usize,
    /// Whether to detect project types (default: true)
    pub detect_projects: bool,
    /// Whether to use incremental indexing (default: true)
    pub incremental: bool,
    /// Collection name override (default: derived from repo)
    pub collection: Option<String>,
}

impl Default for GitIndexingOptions {
    fn default() -> Self {
        Self {
            submodule_depth: 2,
            detect_projects: true,
            incremental: true,
            collection: None,
        }
    }
}

/// Result of git repository indexing
#[derive(Debug, Clone)]
pub struct GitIndexingResult {
    /// Root collection name
    pub collection: String,
    /// Number of files indexed in root
    pub files_indexed: usize,
    /// Number of files skipped (unchanged)
    pub files_skipped: usize,
    /// Submodules discovered and indexed
    pub submodules: Vec<SubmoduleIndexResult>,
    /// Projects detected across all directories
    pub projects: Vec<DetectedProject>,
    /// Total indexing time in milliseconds
    pub duration_ms: u64,
}

/// Result of indexing a single submodule
#[derive(Debug, Clone)]
pub struct SubmoduleIndexResult {
    /// Submodule path relative to parent
    pub path: String,
    /// Collection name for this submodule
    pub collection: String,
    /// Files indexed
    pub files_indexed: usize,
    /// Files skipped (unchanged)
    pub files_skipped: usize,
    /// Projects detected in submodule
    pub projects: Vec<ProjectType>,
}

/// Service for git-aware indexing with submodule and project support
pub struct GitIndexingService<S, P, H>
where
    S: SubmoduleCollector,
    P: ProjectDetectorService,
    H: FileHashService,
{
    submodule_service: Arc<S>,
    project_detector: Arc<P>,
    file_hash_store: Arc<H>,
}

/// Trait for submodule collection (allows mocking)
#[async_trait::async_trait]
pub trait SubmoduleCollector: Send + Sync {
    async fn collect(
        &self,
        repo_path: &Path,
        parent_id: &str,
        max_depth: usize,
    ) -> Result<Vec<SubmoduleInfo>>;
}

/// Trait for project detection (allows mocking)
#[async_trait::async_trait]
pub trait ProjectDetectorService: Send + Sync {
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}

/// Trait for file hash operations (allows mocking)
#[async_trait::async_trait]
pub trait FileHashService: Send + Sync {
    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool>;
    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()>;
    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>>;
    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()>;
    fn compute_hash(path: &Path) -> Result<String>;
}

impl<S, P, H> GitIndexingService<S, P, H>
where
    S: SubmoduleCollector,
    P: ProjectDetectorService,
    H: FileHashService,
{
    /// Create a new GitIndexingService
    #[must_use]
    pub fn new(
        submodule_service: Arc<S>,
        project_detector: Arc<P>,
        file_hash_store: Arc<H>,
    ) -> Self {
        Self {
            submodule_service,
            project_detector,
            file_hash_store,
        }
    }

    /// Index a git repository with full submodule and project detection
    pub async fn index_repository(
        &self,
        repo_path: &Path,
        options: GitIndexingOptions,
    ) -> Result<GitIndexingResult> {
        let start = std::time::Instant::now();

        // Determine collection name
        let collection = options
            .collection
            .clone()
            .unwrap_or_else(|| Self::derive_collection_name(repo_path));

        // Detect projects at root
        let mut all_projects = Vec::new();
        if options.detect_projects {
            let root_projects = self.project_detector.detect_all(repo_path).await;
            for project_type in root_projects {
                all_projects.push(DetectedProject {
                    path: ".".to_string(),
                    project_type,
                    parent_repo_id: None,
                });
            }
        }

        // Index root directory
        let (files_indexed, files_skipped) = if options.incremental {
            self.index_directory_incremental(repo_path, &collection)
                .await?
        } else {
            self.index_directory_full(repo_path, &collection).await?
        };

        // Process submodules (automatic - no opt-in flag)
        // Users control depth via submodule_depth (0 = skip submodules)
        let mut submodule_results = Vec::new();
        if options.submodule_depth > 0 {
            // Generate repo ID (in real impl, would use root commit hash)
            let repo_id = Self::derive_repo_id(repo_path);

            let submodules = self
                .submodule_service
                .collect(repo_path, &repo_id, options.submodule_depth)
                .await?;

            for submodule in submodules {
                let sub_path = repo_path.join(&submodule.path);
                if !sub_path.exists() {
                    tracing::warn!(
                        path = %submodule.path,
                        "Submodule path does not exist, skipping"
                    );
                    continue;
                }

                // Generate hierarchical collection name
                let sub_collection = format!("{}/{}", collection, submodule.path.replace('/', "-"));

                // Detect projects in submodule
                let sub_projects = if options.detect_projects {
                    self.project_detector.detect_all(&sub_path).await
                } else {
                    Vec::new()
                };

                // Add to all projects with parent link
                for project_type in &sub_projects {
                    all_projects.push(DetectedProject {
                        path: submodule.path.clone(),
                        project_type: project_type.clone(),
                        parent_repo_id: Some(repo_id.clone()),
                    });
                }

                // Index submodule
                let (sub_indexed, sub_skipped) = if options.incremental {
                    self.index_directory_incremental(&sub_path, &sub_collection)
                        .await?
                } else {
                    self.index_directory_full(&sub_path, &sub_collection)
                        .await?
                };

                submodule_results.push(SubmoduleIndexResult {
                    path: submodule.path,
                    collection: sub_collection,
                    files_indexed: sub_indexed,
                    files_skipped: sub_skipped,
                    projects: sub_projects,
                });
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(GitIndexingResult {
            collection,
            files_indexed,
            files_skipped,
            submodules: submodule_results,
            projects: all_projects,
            duration_ms,
        })
    }

    /// Derive collection name from repository path
    fn derive_collection_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string()
    }

    /// Derive repository ID (placeholder - real impl uses root commit hash)
    fn derive_repo_id(path: &Path) -> String {
        // In Phase 2, this would use git2 to get root commit hash
        Self::derive_collection_name(path)
    }

    /// Index directory with incremental support (only changed files)
    async fn index_directory_incremental(
        &self,
        path: &Path,
        collection: &str,
    ) -> Result<(usize, usize)> {
        let mut indexed = 0usize;
        let mut skipped = 0usize;

        // Get previously indexed files
        let previously_indexed = self
            .file_hash_store
            .get_indexed_files(collection)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();

        let mut current_files = HashSet::new();

        // Walk directory
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !Self::should_skip_dir(e))
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative = file_path
                .strip_prefix(path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            current_files.insert(relative.clone());

            // Compute hash
            let hash = match H::compute_hash(file_path) {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!(path = %relative, error = %e, "Failed to hash file");
                    continue;
                }
            };

            // Check if changed
            if self
                .file_hash_store
                .has_changed(collection, &relative, &hash)
                .await?
            {
                // TODO: In full integration, call context_service.index_file()
                tracing::debug!(path = %relative, "File changed, re-indexing");
                self.file_hash_store
                    .upsert_hash(collection, &relative, &hash)
                    .await?;
                indexed += 1;
            } else {
                skipped += 1;
            }
        }

        // Mark deleted files as tombstones
        for old_file in previously_indexed {
            if !current_files.contains(&old_file) {
                tracing::debug!(path = %old_file, "File deleted, creating tombstone");
                self.file_hash_store
                    .mark_deleted(collection, &old_file)
                    .await?;
            }
        }

        Ok((indexed, skipped))
    }

    /// Index directory fully (no incremental)
    async fn index_directory_full(&self, path: &Path, collection: &str) -> Result<(usize, usize)> {
        let mut indexed = 0usize;

        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !Self::should_skip_dir(e))
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative = file_path
                .strip_prefix(path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            // Compute and store hash
            if let Ok(hash) = H::compute_hash(file_path) {
                // TODO: In full integration, call context_service.index_file()
                self.file_hash_store
                    .upsert_hash(collection, &relative, &hash)
                    .await?;
                indexed += 1;
            }
        }

        Ok((indexed, 0)) // No skipped files in full index
    }

    /// Check if directory should be skipped
    fn should_skip_dir(entry: &walkdir::DirEntry) -> bool {
        let name = entry.file_name().to_str().unwrap_or("");
        matches!(
            name,
            ".git"
                | "node_modules"
                | "target"
                | "__pycache__"
                | ".venv"
                | "venv"
                | "build"
                | "dist"
                | ".idea"
                | ".vscode"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for testing
    struct MockSubmoduleCollector;

    #[async_trait::async_trait]
    impl SubmoduleCollector for MockSubmoduleCollector {
        async fn collect(
            &self,
            _repo_path: &Path,
            _parent_id: &str,
            _max_depth: usize,
        ) -> Result<Vec<SubmoduleInfo>> {
            Ok(Vec::new())
        }
    }

    struct MockProjectDetector;

    #[async_trait::async_trait]
    impl ProjectDetectorService for MockProjectDetector {
        async fn detect_all(&self, _path: &Path) -> Vec<ProjectType> {
            Vec::new()
        }
    }

    struct MockFileHashService;

    #[async_trait::async_trait]
    impl FileHashService for MockFileHashService {
        async fn has_changed(
            &self,
            _collection: &str,
            _file_path: &str,
            _current_hash: &str,
        ) -> Result<bool> {
            Ok(true) // Always changed for testing
        }

        async fn upsert_hash(
            &self,
            _collection: &str,
            _file_path: &str,
            _hash: &str,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_indexed_files(&self, _collection: &str) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        async fn mark_deleted(&self, _collection: &str, _file_path: &str) -> Result<()> {
            Ok(())
        }

        fn compute_hash(_path: &Path) -> Result<String> {
            Ok("mockhash".to_string())
        }
    }

    #[test]
    fn test_derive_collection_name() {
        let path = Path::new("/home/user/projects/my-repo");
        let name = GitIndexingService::<
            MockSubmoduleCollector,
            MockProjectDetector,
            MockFileHashService,
        >::derive_collection_name(path);
        assert_eq!(name, "my-repo");
    }

    #[test]
    fn test_should_skip_dir() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        std::fs::create_dir(temp.path().join("src")).unwrap();

        let entries: Vec<_> = walkdir::WalkDir::new(temp.path())
            .into_iter()
            .filter_entry(|e| {
                !GitIndexingService::<
                    MockSubmoduleCollector,
                    MockProjectDetector,
                    MockFileHashService,
                >::should_skip_dir(e)
            })
            .filter_map(std::result::Result::ok)
            .collect();

        // Should have temp root and src, but not .git
        let names: Vec<_> = entries
            .iter()
            .map(|e| e.file_name().to_str().unwrap())
            .collect();
        assert!(names.contains(&"src"));
        assert!(!names.contains(&".git"));
    }
}
