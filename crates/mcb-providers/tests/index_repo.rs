//! Integration tests for SeaORM Index Repository.
//!
//! Tests the full index lifecycle: start, progress, complete, fail, clear, stats.

use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::ports::IndexRepository;
use mcb_domain::ports::IndexingOperationStatus;
use mcb_domain::value_objects::CollectionId;
use mcb_providers::database::seaorm::entities::{organization, project};
use mcb_providers::database::seaorm::migration::Migrator;
use mcb_providers::database::seaorm::repos::SeaOrmIndexRepository;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const PROJECT_ID: &str = "proj-idx-001";

async fn setup_db() -> TestResult<Arc<DatabaseConnection>> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(Arc::new(db))
}

async fn seed_org_and_project(db: &DatabaseConnection) -> TestResult {
    let org = organization::ActiveModel {
        id: Set(DEFAULT_ORG_ID.to_owned()),
        name: Set("Default Org".to_owned()),
        slug: Set("default-org".to_owned()),
        settings_json: Set("{}".to_owned()),
        created_at: Set(1_700_000_000),
        updated_at: Set(1_700_000_000),
    };
    org.insert(db).await?;

    let proj = project::ActiveModel {
        id: Set(PROJECT_ID.to_owned()),
        org_id: Set(DEFAULT_ORG_ID.to_owned()),
        name: Set("Index Test Project".to_owned()),
        path: Set("/tmp/index-test".to_owned()),
        created_at: Set(1_700_000_000),
        updated_at: Set(1_700_000_000),
    };
    proj.insert(db).await?;

    Ok(())
}

async fn make_repo(db: &Arc<DatabaseConnection>) -> TestResult<SeaOrmIndexRepository> {
    seed_org_and_project(db.as_ref()).await?;
    Ok(SeaOrmIndexRepository::new(
        Arc::clone(db),
        PROJECT_ID.to_owned(),
    ))
}

// ============================================================================
// Start + Get lifecycle
// ============================================================================

#[tokio::test]
async fn start_indexing_creates_operation() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("test-collection");
    let op_id = repo.start_indexing(&collection, 50).await?;

    let op = repo.get_operation(&op_id).await?;
    assert!(op.is_some(), "operation should exist after start");

    let op = op.unwrap();
    assert_eq!(op.total_files, 50);
    assert_eq!(op.processed_files, 0);
    assert_eq!(op.status, IndexingOperationStatus::Starting);
    assert!(op.current_file.is_none());

    Ok(())
}

// ============================================================================
// Progress tracking
// ============================================================================

#[tokio::test]
async fn update_progress_tracks_files() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("progress-test");
    let op_id = repo.start_indexing(&collection, 100).await?;

    // Update progress
    repo.update_progress(&op_id, Some("src/main.rs".to_owned()), 10)
        .await?;

    let op = repo.get_operation(&op_id).await?.unwrap();
    assert_eq!(op.processed_files, 10);
    assert_eq!(op.current_file.as_deref(), Some("src/main.rs"));
    assert_eq!(op.status, IndexingOperationStatus::InProgress);

    // Update again
    repo.update_progress(&op_id, Some("src/lib.rs".to_owned()), 50)
        .await?;

    let op = repo.get_operation(&op_id).await?.unwrap();
    assert_eq!(op.processed_files, 50);
    assert_eq!(op.current_file.as_deref(), Some("src/lib.rs"));

    Ok(())
}

// ============================================================================
// Complete operation
// ============================================================================

#[tokio::test]
async fn complete_operation_sets_terminal_state() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("complete-test");
    let op_id = repo.start_indexing(&collection, 10).await?;

    repo.complete_operation(&op_id).await?;

    let op = repo.get_operation(&op_id).await?.unwrap();
    assert_eq!(op.status, IndexingOperationStatus::Completed);

    // Should no longer be active
    let active = repo.get_active_operation(&collection).await?;
    assert!(active.is_none(), "completed op should not be active");

    Ok(())
}

// ============================================================================
// Fail operation
// ============================================================================

#[tokio::test]
async fn fail_operation_records_error() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("fail-test");
    let op_id = repo.start_indexing(&collection, 10).await?;

    repo.fail_operation(&op_id, "disk full").await?;

    let op = repo.get_operation(&op_id).await?.unwrap();
    assert!(
        matches!(op.status, IndexingOperationStatus::Failed(ref msg) if msg == "disk full"),
        "expected Failed(disk full), got {:?}",
        op.status
    );

    // Should no longer be active
    let active = repo.get_active_operation(&collection).await?;
    assert!(active.is_none(), "failed op should not be active");

    Ok(())
}

// ============================================================================
// Active operation detection
// ============================================================================

#[tokio::test]
async fn get_active_operation_finds_running() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("active-test");

    // No active operation initially
    let active = repo.get_active_operation(&collection).await?;
    assert!(active.is_none());

    // Start one
    let op_id = repo.start_indexing(&collection, 20).await?;

    // Should be active
    let active = repo.get_active_operation(&collection).await?;
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, op_id);

    Ok(())
}

// ============================================================================
// List operations
// ============================================================================

#[tokio::test]
async fn list_operations_returns_all() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let c1 = CollectionId::from_name("list-test-1");
    let c2 = CollectionId::from_name("list-test-2");

    repo.start_indexing(&c1, 10).await?;
    repo.start_indexing(&c2, 20).await?;

    let ops = repo.list_operations().await?;
    assert_eq!(ops.len(), 2);

    Ok(())
}

// ============================================================================
// Clear index
// ============================================================================

#[tokio::test]
async fn clear_index_removes_data_and_cancels_active() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("clear-test");

    // Start an operation
    let op_id = repo.start_indexing(&collection, 10).await?;
    repo.update_progress(&op_id, Some("file.rs".to_owned()), 5)
        .await?;

    // Seed some file hashes via raw SQL (simulating indexed files)
    let collection_str = collection.as_str();
    let collection_id_str = format!("{PROJECT_ID}:{collection_str}");

    db.execute_unprepared(&format!(
        "INSERT INTO collections (id, project_id, name, vector_name, created_at) \
         VALUES ('{collection_id_str}', '{PROJECT_ID}', '{collection_str}', '{collection_str}', 1700000000)"
    ))
    .await?;

    db.execute_unprepared(&format!(
        "INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at) \
         VALUES ('{PROJECT_ID}', '{collection_str}', 'src/main.rs', 'abc123', 1700000001)"
    ))
    .await?;

    db.execute_unprepared(&format!(
        "INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at) \
         VALUES ('{PROJECT_ID}', '{collection_str}', 'src/lib.rs', 'def456', 1700000002)"
    ))
    .await?;

    // Clear the index
    let removed = repo.clear_index(&collection).await?;
    assert_eq!(removed, 2, "should have removed 2 file hashes");

    // Active operation should be cancelled
    let op = repo.get_operation(&op_id).await?.unwrap();
    assert!(
        matches!(op.status, IndexingOperationStatus::Failed(ref msg) if msg == "index cleared"),
        "active op should be cancelled, got {:?}",
        op.status
    );

    // No more active operations
    let active = repo.get_active_operation(&collection).await?;
    assert!(active.is_none());

    Ok(())
}

// ============================================================================
// Index stats
// ============================================================================

#[tokio::test]
async fn get_index_stats_reports_correctly() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let collection = CollectionId::from_name("stats-test");
    let collection_str = collection.as_str();

    // Initially empty
    let stats = repo.get_index_stats(&collection).await?;
    assert_eq!(stats.indexed_files, 0);
    assert!(stats.last_indexed_at.is_none());
    assert!(!stats.is_indexing);

    // Start indexing
    let op_id = repo.start_indexing(&collection, 5).await?;

    let stats = repo.get_index_stats(&collection).await?;
    assert!(stats.is_indexing, "should show indexing in progress");

    // Complete it
    repo.complete_operation(&op_id).await?;

    let stats = repo.get_index_stats(&collection).await?;
    assert!(!stats.is_indexing);
    assert!(stats.last_indexed_at.is_some());

    // Add some file hashes
    db.execute_unprepared(&format!(
        "INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at) \
         VALUES ('{PROJECT_ID}', '{collection_str}', 'a.rs', 'h1', 1700000001)"
    ))
    .await?;

    db.execute_unprepared(&format!(
        "INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at) \
         VALUES ('{PROJECT_ID}', '{collection_str}', 'b.rs', 'h2', 1700000002)"
    ))
    .await?;

    // One tombstoned
    db.execute_unprepared(&format!(
        "INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at, deleted_at) \
         VALUES ('{PROJECT_ID}', '{collection_str}', 'c.rs', 'h3', 1700000003, 1700000004)"
    ))
    .await?;

    let stats = repo.get_index_stats(&collection).await?;
    assert_eq!(stats.indexed_files, 2, "tombstoned file should not count");

    Ok(())
}

// ============================================================================
// Error handling: operation not found
// ============================================================================

#[tokio::test]
async fn update_progress_on_missing_op_returns_not_found() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let fake_id = mcb_domain::value_objects::OperationId::new();
    let result = repo.update_progress(&fake_id, None, 0).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(
        matches!(err, mcb_domain::error::Error::NotFound { .. }),
        "expected NotFound, got: {err:?}"
    );

    Ok(())
}

#[tokio::test]
async fn complete_missing_op_returns_not_found() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let fake_id = mcb_domain::value_objects::OperationId::new();
    let result = repo.complete_operation(&fake_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        mcb_domain::error::Error::NotFound { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn fail_missing_op_returns_not_found() -> TestResult {
    let db = setup_db().await?;
    let repo = make_repo(&db).await?;

    let fake_id = mcb_domain::value_objects::OperationId::new();
    let result = repo.fail_operation(&fake_id, "oops").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        mcb_domain::error::Error::NotFound { .. }
    ));

    Ok(())
}
