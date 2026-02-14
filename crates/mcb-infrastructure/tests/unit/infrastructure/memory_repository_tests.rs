use std::sync::Arc;

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::value_objects::ObservationId;
use mcb_providers::database::{create_memory_repository, create_memory_repository_with_executor};
use rstest::rstest;

use rstest::*;
use tempfile::TempDir;

use crate::test_utils::create_test_project;

#[fixture]
async fn repo() -> (Arc<dyn MemoryRepository>, TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let repo = create_memory_repository(db_path).await.unwrap();
    (repo, temp_dir)
}

#[fixture]
async fn repo_and_executor() -> (
    Arc<dyn MemoryRepository>,
    Arc<dyn DatabaseExecutor>,
    TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .unwrap();
    (repo, executor, temp_dir)
}

#[rstest]
#[tokio::test]
async fn test_memory_repository_creates(#[future] repo: (Arc<dyn MemoryRepository>, TempDir)) {
    let (repo, _dir) = repo.await;
    let results = repo.search("test", 1).await.unwrap();
    assert!(results.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_memory_repository_store_and_get_observation(
    #[future] repo_and_executor: (
        Arc<dyn MemoryRepository>,
        Arc<dyn DatabaseExecutor>,
        TempDir,
    ),
) {
    let (repo, executor, _dir) = repo_and_executor.await;

    let project_id = "test-project";
    create_test_project(executor.as_ref(), project_id).await;

    let obs_id = ObservationId::from_name("test-obs-1");
    let obs = Observation {
        id: obs_id.to_string(),
        project_id: project_id.to_string(),
        content: "content".to_string(),
        content_hash: "hash1".to_string(),
        tags: vec![],
        r#type: ObservationType::Context,
        metadata: Default::default(),
        created_at: 0,
        embedding_id: None,
    };

    repo.store_observation(&obs).await.unwrap();

    let got = repo.get_observation(&obs_id).await.unwrap();

    assert!(got.is_some());
    assert_eq!(got.unwrap().content, "content");
}
