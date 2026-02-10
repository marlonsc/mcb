use mcb_infrastructure::database::memory_provider::MemoryDatabaseProvider;

#[tokio::test]
async fn test_connect_file_based() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let pool = MemoryDatabaseProvider::connect(db_path)
        .await
        .expect("create file-based pool");
    assert!(!pool.is_closed());
}
