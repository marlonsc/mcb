use std::sync::Arc;

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::value_objects::ObservationId;
use mcb_providers::database::create_memory_repository_with_executor;
use rstest::rstest;
use uuid::Uuid;

use rstest::*;
use tempfile::TempDir;

use crate::test_utils::create_test_project;

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
async fn test_fts_search_flow(
    #[future] repo_and_executor: (
        Arc<dyn MemoryRepository>,
        Arc<dyn DatabaseExecutor>,
        TempDir,
    ),
) {
    let (repo, executor, _dir) = repo_and_executor.await;

    let project_id = "test-project".to_string();
    create_test_project(executor.as_ref(), &project_id).await;

    let id = Uuid::new_v4().to_string();
    let obs = Observation {
        id: id.clone(),
        project_id: project_id.clone(),
        content: "The quick brown fox".to_string(),
        content_hash: "hash1".to_string(),
        tags: vec![],
        r#type: ObservationType::Context,
        metadata: Default::default(),
        created_at: 100,
        embedding_id: None,
    };
    repo.store_observation(&obs).await.unwrap();

    // 2. Search FTS for "fox" -> returns ID with rank
    let results = repo.search("fox", 10).await.unwrap();
    assert!(results.iter().any(|r| r.id == id), "Should find 'fox'");

    // 3. Search FTS for "dog" -> returns empty
    let results = repo.search("dog", 10).await.unwrap();
    assert!(results.is_empty(), "Should not find 'dog'");

    // 4. Delete observation
    repo.delete_observation(&ObservationId::new(&id))
        .await
        .unwrap();

    // 5. Search FTS for "fox" -> returns empty (verifies trigger)
    let results = repo.search("fox", 10).await.unwrap();
    assert!(results.is_empty(), "Should not find 'fox' after delete");
}
