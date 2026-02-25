use std::sync::Arc;

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::value_objects::ObservationId;
use mcb_infrastructure::di::repositories::create_memory_repository_with_db;
use sea_orm::DatabaseConnection;
use tempfile::TempDir;
use uuid::Uuid;

use crate::utils::create_test_project;

async fn setup_repo_and_db()
-> Result<(Arc<dyn MemoryRepository>, Arc<DatabaseConnection>, TempDir), Box<dyn std::error::Error>>
{
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let (repo, db) = create_memory_repository_with_db(db_path).await?;
    Ok((repo, db, temp_dir))
}

#[tokio::test]
async fn test_fts_search_flow() -> Result<(), Box<dyn std::error::Error>> {
    let (repo, db, _dir) = setup_repo_and_db().await?;

    let project_id = "test-project".to_owned();
    create_test_project(db.as_ref(), &project_id).await?;

    let id = Uuid::new_v4().to_string();
    let obs = Observation {
        id: id.clone(),
        project_id: project_id.clone(),
        content: "The quick brown fox".to_owned(),
        content_hash: "hash1".to_owned(),
        tags: vec![],
        r#type: ObservationType::Context,
        metadata: Default::default(),
        created_at: 100,
        embedding_id: None,
    };
    repo.store_observation(&obs).await?;

    // 2. Search FTS for "fox" -> returns ID with rank
    let results = repo.search("fox", 10).await?;
    assert!(results.iter().any(|r| r.id == id), "Should find 'fox'");

    // 3. Search FTS for "dog" -> returns empty
    let results = repo.search("dog", 10).await?;
    assert!(results.is_empty(), "Should not find 'dog'");

    // 4. Delete observation
    repo.delete_observation(&ObservationId::from(id.as_str()))
        .await?;

    // 5. Search FTS for "fox" -> returns empty (verifies trigger)
    let results = repo.search("fox", 10).await?;
    assert!(results.is_empty(), "Should not find 'fox' after delete");
    Ok(())
}
