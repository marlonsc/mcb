use std::sync::Arc;

use mcb_domain::entities::memory::{Observation, ObservationType};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::value_objects::ObservationId;
use mcb_providers::database::{create_memory_repository, create_memory_repository_with_executor};
use tempfile::TempDir;

use crate::test_utils::create_test_project;

async fn setup_repo() -> Result<(Arc<dyn MemoryRepository>, TempDir), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let repo = create_memory_repository(db_path).await?;
    Ok((repo, temp_dir))
}

async fn setup_repo_and_executor() -> Result<
    (
        Arc<dyn MemoryRepository>,
        Arc<dyn DatabaseExecutor>,
        TempDir,
    ),
    Box<dyn std::error::Error>,
> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let (repo, executor) = create_memory_repository_with_executor(db_path).await?;
    Ok((repo, executor, temp_dir))
}

#[tokio::test]
async fn test_memory_repository_creates() -> Result<(), Box<dyn std::error::Error>> {
    let (repo, _dir) = setup_repo().await?;
    let results = repo.search("test", 1).await?;
    assert!(results.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_memory_repository_store_and_get_observation() -> Result<(), Box<dyn std::error::Error>>
{
    let (repo, executor, _dir) = setup_repo_and_executor().await?;

    let project_id = "test-project";
    create_test_project(executor.as_ref(), project_id).await?;

    let obs_id = ObservationId::from_name("test-obs-1");
    let obs = Observation {
        id: obs_id.to_string(),
        project_id: project_id.to_owned(),
        content: "content".to_owned(),
        content_hash: "hash1".to_owned(),
        tags: vec![],
        r#type: ObservationType::Context,
        metadata: Default::default(),
        created_at: 0,
        embedding_id: None,
    };

    repo.store_observation(&obs).await?;

    let got = repo.get_observation(&obs_id).await?;

    assert!(got.is_some());
    let observation = got.ok_or("observation should exist")?;
    assert_eq!(observation.content, "content");
    Ok(())
}
