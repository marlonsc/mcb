#![allow(unsafe_code)]

use rstest::rstest;
use serial_test::serial;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use mcb_domain::ports::FileHashRepository;
use mcb_domain::value_objects::CollectionId;

use crate::utils::shared_context::shared_app_context;
use rstest::*;
use tempfile::NamedTempFile;

#[fixture]
fn file_hash_repo() -> Arc<dyn FileHashRepository> {
    let ctx = shared_app_context();
    ctx.file_hash_repository()
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_has_changed(file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo;

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
#[serial]
async fn test_tombstone(file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo;

    repo.upsert_hash("test", "file.rs", "hash").await.unwrap();
    repo.mark_deleted("test", "file.rs").await.unwrap();

    // Should not be found after tombstone
    let hash = repo.get_hash("test", "file.rs").await.unwrap();
    assert!(hash.is_none());
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_resurrect_after_tombstone(file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo;

    repo.upsert_hash("test", "file.rs", "hash1").await.unwrap();
    repo.mark_deleted("test", "file.rs").await.unwrap();

    // Upsert clears tombstone
    repo.upsert_hash("test", "file.rs", "hash2").await.unwrap();

    let hash = repo.get_hash("test", "file.rs").await.unwrap();
    assert_eq!(hash, Some("hash2".to_owned()));
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_get_indexed_files(file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo;

    repo.upsert_hash("col", "a.rs", "h1").await.unwrap();
    repo.upsert_hash("col", "b.rs", "h2").await.unwrap();
    repo.upsert_hash("col", "c.rs", "h3").await.unwrap();
    repo.mark_deleted("col", "b.rs").await.unwrap();

    let files = repo.get_indexed_files("col").await.unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"a.rs".to_owned()));
    assert!(files.contains(&"c.rs".to_owned()));
    assert!(!files.contains(&"b.rs".to_owned()));
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_compute_file_hash(file_hash_repo: Arc<dyn FileHashRepository>) {
    let repo = file_hash_repo;
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
#[serial]
async fn test_indexing_persists_file_hash_metadata() {
    let ctx = shared_app_context();
    let services = ctx.build_domain_services().await.unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("a.rs"), "fn a() {}\n").unwrap();
    std::fs::write(temp_dir.path().join("b.rs"), "fn b() {}\n").unwrap();

    let collection = CollectionId::from_name("index-persistence-test");
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

    let collection_str = collection.to_string();
    let indexed_files = ctx
        .file_hash_repository()
        .get_indexed_files(&collection_str)
        .await
        .unwrap();
    assert!(
        indexed_files.len() >= 2,
        "expected indexed file metadata to be persisted"
    );

    let (collections, _chunks) = services.context_service.get_stats().await.unwrap();
    assert!(collections >= 1, "expected at least one indexed collection");
}
