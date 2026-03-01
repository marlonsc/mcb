#[cfg(test)]
mod tests {
    use rstest::rstest;
    use sqlx::sqlite::SqlitePool;

    #[rstest]
    #[tokio::test]
    async fn test_fts5_availability() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Try creating an FTS5 table
        let result = sqlx::query("CREATE VIRTUAL TABLE test_fts USING fts5(content)")
            .execute(&pool)
            .await;

        match result {
            Ok(_) => println!("FTS5 is available"),
            Err(e) => panic!("FTS5 check failed: {e}"),
        }

        // Try a simple insert and match
        sqlx::query("INSERT INTO test_fts (content) VALUES ('hello world')")
            .execute(&pool)
            .await
            .unwrap();

        let row: (i64,) =
            sqlx::query_as("SELECT count(*) FROM test_fts WHERE test_fts MATCH 'hello'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(row.0, 1);
    }
}
