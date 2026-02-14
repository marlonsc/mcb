use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use mcb_domain::ports::repositories::FileHashRepository;
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::config::types::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use rstest::*;
use tempfile::NamedTempFile;

#[fixture]
async fn file_hash_repo() -> Arc<dyn FileHashRepository> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_path = std::env::temp_dir().join(format!("mcb-file-hash-tests-{unique}.db"));

    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(db_path);

    let ctx = init_app(config).await.unwrap();
    ctx.file_hash_repository()
}

#[rstest]
#[tokio::test]
async fn test_has_changed(#[future] file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo.await;

    // New file
    assert!(repo.has_changed("test", "new.rs", "hash1").await.unwrap());

    // Insert
    repo.upsert_hash("test", "new.rs", "hash1").await.unwrap();

    // Same hash
    assert!(!repo.has_changed("test", "new.rs", "hash1").await.unwrap());

    // Different hash
    assert!(repo.has_changed("test", "new.rs", "hash2").await.unwrap());
}

#[rstest]
#[tokio::test]
async fn test_tombstone(#[future] file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo.await;

    repo.upsert_hash("test", "file.rs", "hash").await.unwrap();
    repo.mark_deleted("test", "file.rs").await.unwrap();

    // Should not be found after tombstone
    let hash = repo.get_hash("test", "file.rs").await.unwrap();
    assert!(hash.is_none());
}

#[rstest]
#[tokio::test]
async fn test_resurrect_after_tombstone(#[future] file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo.await;

    repo.upsert_hash("test", "file.rs", "hash1").await.unwrap();
    repo.mark_deleted("test", "file.rs").await.unwrap();

    // Upsert clears tombstone
    repo.upsert_hash("test", "file.rs", "hash2").await.unwrap();

    let hash = repo.get_hash("test", "file.rs").await.unwrap();
    assert_eq!(hash, Some("hash2".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_get_indexed_files(#[future] file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo.await;

    repo.upsert_hash("col", "a.rs", "h1").await.unwrap();
    repo.upsert_hash("col", "b.rs", "h2").await.unwrap();
    repo.upsert_hash("col", "c.rs", "h3").await.unwrap();
    repo.mark_deleted("col", "b.rs").await.unwrap();

    let files = repo.get_indexed_files("col").await.unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"a.rs".to_string()));
    assert!(files.contains(&"c.rs".to_string()));
    assert!(!files.contains(&"b.rs".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_compute_file_hash(#[future] file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo.await;
    let mut temp = NamedTempFile::new().unwrap();
    write!(temp, "Hello, World!").unwrap();

    let hash = repo.compute_hash(temp.path()).unwrap();
    // SHA-256 of "Hello, World!"
    assert_eq!(
        hash,
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
    );
}

#[rstest]
#[tokio::test]
async fn test_indexing_persists_file_hash_metadata() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_path = std::env::temp_dir().join(format!("mcb-indexing-tests-{unique}.db"));

    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(db_path);

    let ctx = init_app(config).await.unwrap();
    let services = ctx.build_domain_services().await.unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("a.rs"), "fn a() {}\n").unwrap();
    std::fs::write(temp_dir.path().join("b.rs"), "fn b() {}\n").unwrap();

    let collection = CollectionId::new("index-persistence-test");
    let result = services
        .indexing_service
        .index_codebase(temp_dir.path(), &collection)
        .await
        .unwrap();
    assert_eq!(result.status, "started");

    for _ in 0..50 {
        if !services.indexing_service.get_status().is_indexing {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let indexed_files = ctx
        .file_hash_repository()
        .get_indexed_files(collection.as_str())
        .await
        .unwrap();
    assert!(
        indexed_files.len() >= 2,
        "expected indexed file metadata to be persisted"
    );

    let (collections, _chunks) = services.context_service.get_stats().await.unwrap();
    assert!(collections >= 1, "expected at least one indexed collection");
}
