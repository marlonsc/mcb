use std::io::Write;
use std::sync::Arc;

use mcb_domain::ports::repositories::FileHashRepository;
use mcb_infrastructure::config::types::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use tempfile::NamedTempFile;

async fn create_store() -> Arc<dyn FileHashRepository> {
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

#[tokio::test]
async fn test_has_changed() {
    let store = create_store().await;

    // New file
    assert!(store.has_changed("test", "new.rs", "hash1").await.unwrap());

    // Insert
    store.upsert_hash("test", "new.rs", "hash1").await.unwrap();

    // Same hash
    assert!(!store.has_changed("test", "new.rs", "hash1").await.unwrap());

    // Different hash
    assert!(store.has_changed("test", "new.rs", "hash2").await.unwrap());
}

#[tokio::test]
async fn test_tombstone() {
    let store = create_store().await;

    store.upsert_hash("test", "file.rs", "hash").await.unwrap();
    store.mark_deleted("test", "file.rs").await.unwrap();

    // Should not be found after tombstone
    let hash = store.get_hash("test", "file.rs").await.unwrap();
    assert!(hash.is_none());
}

#[tokio::test]
async fn test_resurrect_after_tombstone() {
    let store = create_store().await;

    store.upsert_hash("test", "file.rs", "hash1").await.unwrap();
    store.mark_deleted("test", "file.rs").await.unwrap();

    // Upsert clears tombstone
    store.upsert_hash("test", "file.rs", "hash2").await.unwrap();

    let hash = store.get_hash("test", "file.rs").await.unwrap();
    assert_eq!(hash, Some("hash2".to_string()));
}

#[tokio::test]
async fn test_get_indexed_files() {
    let store = create_store().await;

    store.upsert_hash("col", "a.rs", "h1").await.unwrap();
    store.upsert_hash("col", "b.rs", "h2").await.unwrap();
    store.upsert_hash("col", "c.rs", "h3").await.unwrap();
    store.mark_deleted("col", "b.rs").await.unwrap();

    let files = store.get_indexed_files("col").await.unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"a.rs".to_string()));
    assert!(files.contains(&"c.rs".to_string()));
    assert!(!files.contains(&"b.rs".to_string()));
}

#[tokio::test]
async fn test_compute_file_hash() {
    let store = create_store().await;
    let mut temp = NamedTempFile::new().unwrap();
    write!(temp, "Hello, World!").unwrap();

    let hash = store.compute_hash(temp.path()).unwrap();
    // SHA-256 of "Hello, World!"
    assert_eq!(
        hash,
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
    );
}
