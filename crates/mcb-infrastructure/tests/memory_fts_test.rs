use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_infrastructure::repositories::memory_repository::SqliteMemoryRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_fts_search_flow() {
    let repo = SqliteMemoryRepository::in_memory().await.unwrap();

    // 1. Insert observation "The quick brown fox"
    let id = Uuid::new_v4().to_string();
    let obs = Observation {
        id: id.clone(),
        content: "The quick brown fox".to_string(),
        content_hash: "hash1".to_string(),
        tags: vec![],
        observation_type: ObservationType::Context,
        metadata: Default::default(),
        created_at: 100,
        embedding_id: None,
    };
    repo.store_observation(&obs).await.unwrap();

    // 2. Search FTS for "fox" -> returns ID
    let results = repo.search_fts("fox", 10).await.unwrap();
    assert!(results.contains(&id), "Should find 'fox'");

    // 3. Search FTS for "dog" -> returns empty
    let results = repo.search_fts("dog", 10).await.unwrap();
    assert!(results.is_empty(), "Should not find 'dog'");

    // 4. Delete observation
    repo.delete_observation(&id).await.unwrap();

    // 5. Search FTS for "fox" -> returns empty (verifies trigger)
    let results = repo.search_fts("fox", 10).await.unwrap();
    assert!(results.is_empty(), "Should not find 'fox' after delete");
}
