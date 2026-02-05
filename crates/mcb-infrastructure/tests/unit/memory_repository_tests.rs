use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_providers::database::create_memory_repository_in_memory;
use std::sync::Arc;

async fn create_test_project(executor: &dyn DatabaseExecutor, project_id: &str) {
    let now = chrono::Utc::now().timestamp();
    executor
        .execute(
            "INSERT INTO projects (id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            &[
                SqlParam::String(project_id.to_string()),
                SqlParam::String(project_id.to_string()),
                SqlParam::String("/test".to_string()),
                SqlParam::I64(now),
                SqlParam::I64(now),
            ],
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_memory_repository_in_memory_creates() {
    let repo: Arc<dyn MemoryRepository> = create_memory_repository_in_memory().await.unwrap();
    let results = repo.search_fts("test", 1).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_memory_repository_store_and_get_observation() {
    let (repo, executor): (Arc<dyn MemoryRepository>, Arc<dyn DatabaseExecutor>) =
        mcb_providers::database::create_memory_repository_in_memory_with_executor()
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
        observation_type: ObservationType::Context,
        metadata: Default::default(),
        created_at: 0,
        embedding_id: None,
    };
    repo.store_observation(&obs).await.unwrap();
    let got = repo.get_observation("id1").await.unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().content, "content");
}
