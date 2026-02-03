use std::path::Path;

use async_trait::async_trait;
use tempfile::TempDir;
use walkdir::WalkDir;

use mcb_application::use_cases::vcs_indexing::{SubmoduleCollector, VcsIndexingService};
use mcb_domain::entities::project::ProjectType;
use mcb_domain::entities::submodule::SubmoduleInfo;
use mcb_domain::error::Result;
use mcb_domain::ports::services::{FileHashService, ProjectDetectorService};

struct MockSubmoduleCollector;

#[async_trait]
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

#[async_trait]
impl ProjectDetectorService for MockProjectDetector {
    async fn detect_all(&self, _path: &Path) -> Vec<ProjectType> {
        Vec::new()
    }
}

struct MockFileHashService;

#[async_trait]
impl FileHashService for MockFileHashService {
    async fn has_changed(
        &self,
        _collection: &str,
        _file_path: &str,
        _current_hash: &str,
    ) -> Result<bool> {
        Ok(true)
    }

    async fn upsert_hash(&self, _collection: &str, _file_path: &str, _hash: &str) -> Result<()> {
        Ok(())
    }

    async fn get_indexed_files(&self, _collection: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn mark_deleted(&self, _collection: &str, _file_path: &str) -> Result<()> {
        Ok(())
    }

    fn compute_hash(_path: &Path) -> Result<String> {
        Ok("mockhash".to_owned())
    }
}

#[test]
fn test_derive_collection_name() {
    let path = Path::new("/home/user/projects/my-repo");
    let name = VcsIndexingService::<
        MockSubmoduleCollector,
        MockProjectDetector,
        MockFileHashService,
    >::derive_collection_name(path);
    assert_eq!(name, "my-repo");
}

#[test]
fn test_should_skip_dir() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    std::fs::create_dir(temp.path().join(".git")).unwrap();
    std::fs::create_dir(temp.path().join("src")).unwrap();

    let entries: Vec<_> = WalkDir::new(temp.path())
        .into_iter()
        .filter_entry(|e| {
            !VcsIndexingService::<
                MockSubmoduleCollector,
                MockProjectDetector,
                MockFileHashService,
            >::should_skip_dir(e)
        })
        .filter_map(std::result::Result::ok)
        .collect();

    let names: Vec<_> = entries
        .iter()
        .map(|e| e.file_name().to_str().unwrap())
        .collect();
    assert!(names.contains(&"src"));
    assert!(!names.contains(&".git"));
}
