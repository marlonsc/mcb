use std::sync::Arc;

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::value_objects::ObservationId;
use uuid::Uuid;

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
async fn test_fts_search_flow() {
    let (repo, executor): (Arc<dyn MemoryRepository>, Arc<dyn DatabaseExecutor>) =
        mcb_providers::database::create_memory_repository_in_memory_with_executor()
            .await
            .unwrap();

    let project_id = "test-project".to_string();
    create_test_project(executor.as_ref(), &project_id).await;

    let id = Uuid::new_v4().to_string();
    let obs = Observation {
        id: id.clone(),
        project_id: project_id.clone(),
        content: "The quick brown fox".to_string(),
        content_hash: "hash1".to_string(),
        tags: vec![],
        observation_type: ObservationType::Context,
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
