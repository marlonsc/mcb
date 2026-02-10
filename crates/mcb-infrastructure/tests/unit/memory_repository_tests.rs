use std::sync::Arc;

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::value_objects::ObservationId;
use mcb_providers::database::{create_memory_repository, create_memory_repository_with_executor};

use super::test_utils::create_test_project;

#[tokio::test]
async fn test_memory_repository_creates() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let repo: Arc<dyn MemoryRepository> = create_memory_repository(db_path).await.unwrap();
    let results = repo.search("test", 1).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_memory_repository_store_and_get_observation() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let (repo, executor): (Arc<dyn MemoryRepository>, Arc<dyn DatabaseExecutor>) =
        create_memory_repository_with_executor(db_path)
            .await
            .unwrap();

    let project_id = "test-project";
    create_test_project(executor.as_ref(), project_id).await;

    let obs = Observation {
        id: "id1".to_string(),
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
    let got = repo
        .get_observation(&ObservationId::new("id1"))
        .await
        .unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().content, "content");
}
