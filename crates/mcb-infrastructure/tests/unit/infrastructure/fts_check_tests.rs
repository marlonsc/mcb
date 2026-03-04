use rstest::{fixture, rstest};
use sqlx::sqlite::SqlitePool;

#[fixture]
async fn pool() -> SqlitePool {
    SqlitePool::connect(mcb_utils::constants::SQLITE_MEMORY_DSN)
        .await
        .expect("failed to connect to memory sqlite")
}

#[rstest]
#[tokio::test]
async fn test_fts5_availability(#[future] pool: SqlitePool) {
    let pool = pool.await;
    // Try creating an FTS5 table
    sqlx::query("CREATE VIRTUAL TABLE test_fts USING fts5(content)")
        .execute(&pool)
        .await
        .expect("FTS5 table creation failed - FTS5 may not be available");

    // Try a simple insert and match
    sqlx::query("INSERT INTO test_fts (content) VALUES ('hello world')")
        .execute(&pool)
        .await
        .expect("FTS5 insert failed");

    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM test_fts WHERE test_fts MATCH 'hello'")
            .fetch_one(&pool)
            .await
            .expect("FTS5 match query failed");

    assert_eq!(count, 1);
}
