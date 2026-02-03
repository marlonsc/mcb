//! Tests for memory repository (REF003: dedicated test file).

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_infrastructure::repositories::memory_repository::SqliteMemoryRepository;

#[tokio::test]
async fn test_memory_repository_in_memory_creates() {
    let repo = SqliteMemoryRepository::in_memory().await.unwrap();
    let results = repo.search_fts("test", 1).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_memory_repository_store_and_get_observation() {
    let repo = SqliteMemoryRepository::in_memory().await.unwrap();
    let obs = Observation {
        id: "id1".to_string(),
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
