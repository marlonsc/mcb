//! File hash storage for incremental indexing.
//!
//! Stores SHA-256 content hashes in SQLite to detect changed files.
//! Uses tombstones for soft-delete with configurable TTL cleanup.

use std::io::{BufReader, Read};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

use mcb_domain::error::{Error, Result};

/// Configuration for FileHashStore
///
/// Default uses persistent SQLite at `~/.mcb/metaproject.db`.
/// For testing, use `FileHashConfig::in_memory()`.
/// Postgres support available via DI (future phase).
#[derive(Debug, Clone)]
pub struct FileHashConfig {
    /// Database URL (sqlite:path or :memory:)
    /// Default: ~/.mcb/metaproject.db (persistent storage)
    pub database_url: String,
    /// Tombstone TTL in seconds (default: 30 days = 2592000)
    pub tombstone_ttl_seconds: i64,
    /// Maximum pool connections
    pub max_connections: u32,
}

impl Default for FileHashConfig {
    fn default() -> Self {
        // Default to persistent storage in user's home directory
        // This preserves incremental indexing state across restarts
        let default_path = dirs::home_dir()
            .map(|h| h.join(".mcb").join("metaproject.db"))
            .unwrap_or_else(|| std::path::PathBuf::from("metaproject.db"));

        Self {
            database_url: format!("sqlite:{}", default_path.display()),
            tombstone_ttl_seconds: 30 * 24 * 60 * 60, // 30 days
            max_connections: 5,
        }
    }
}

impl FileHashConfig {
    /// Create config with in-memory database (for testing only)
    ///
    /// WARNING: Data is lost on process exit. Use only for tests.
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            database_url: "sqlite::memory:".to_string(),
            ..Default::default()
        }
    }

    /// Create config with file-based SQLite database
    #[must_use]
    pub fn with_file(path: &Path) -> Self {
        Self {
            database_url: format!("sqlite:{}", path.display()),
            ..Default::default()
        }
    }
}

/// Record representing a file's hash state
#[derive(Debug, Clone)]
pub struct FileHashRecord {
    /// Collection the file belongs to
    pub collection: String,
    /// File path relative to collection root
    pub file_path: String,
    /// SHA-256 content hash
    pub content_hash: String,
    /// Timestamp when indexed (Unix epoch seconds)
    pub indexed_at: i64,
    /// Tombstone timestamp (None = active, Some = deleted)
    pub deleted_at: Option<i64>,
}

/// Store for tracking file content hashes
pub struct FileHashStore {
    pool: SqlitePool,
    config: FileHashConfig,
}

impl FileHashStore {
    /// Create a new FileHashStore with given configuration
    pub async fn new(config: FileHashConfig) -> Result<Self> {
        // Ensure parent directory exists for file-based databases
        if let Some(path_str) = config.database_url.strip_prefix("sqlite:") {
            if path_str != ":memory:" {
                let path = Path::new(path_str);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        Error::internal(format!("Failed to create database directory: {e}"))
                    })?;
                }
            }
        }

        let options: SqliteConnectOptions = config
            .database_url
            .parse()
            .map_err(|e| Error::internal(format!("Invalid database URL: {e}")))?;

        let options = options.create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await
            .map_err(|e| Error::database(format!("Failed to connect to database: {e}")))?;

        let store = Self { pool, config };
        store.run_migrations().await?;

        Ok(store)
    }

    /// Create with in-memory database (for testing only)
    ///
    /// WARNING: Data is lost on process exit. Use only for tests.
    pub async fn in_memory() -> Result<Self> {
        Self::new(FileHashConfig::in_memory()).await
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_hashes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collection TEXT NOT NULL,
                file_path TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                indexed_at INTEGER NOT NULL,
                deleted_at INTEGER,
                UNIQUE(collection, file_path)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to create file_hashes table: {e}")))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_file_hashes_collection ON file_hashes(collection)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to create collection index: {e}")))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_file_hashes_deleted ON file_hashes(deleted_at)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to create deleted_at index: {e}")))?;

        Ok(())
    }

    /// Compute SHA-256 hash of file content
    pub fn compute_file_hash(path: &Path) -> Result<String> {
        let file = std::fs::File::open(path)
            .map_err(|e| Error::io(format!("Failed to open file {path:?}: {e}")))?;

        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();

        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .map_err(|e| Error::io(format!("Failed to read file {path:?}: {e}")))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get current Unix timestamp
    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Get hash for a file (returns None if not found or tombstoned)
    pub async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT content_hash FROM file_hashes WHERE collection = ? AND file_path = ? AND deleted_at IS NULL",
        )
        .bind(collection)
        .bind(file_path)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to get hash: {e}")))?;

        Ok(result.map(|(hash,)| hash))
    }

    /// Check if file has changed (returns true if new or hash differs)
    pub async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool> {
        match self.get_hash(collection, file_path).await? {
            Some(stored_hash) => Ok(stored_hash != current_hash),
            None => Ok(true), // New file
        }
    }

    /// Upsert hash for a file (insert or update)
    pub async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()> {
        let now = Self::now();

        sqlx::query(
            r#"
            INSERT INTO file_hashes (collection, file_path, content_hash, indexed_at, deleted_at)
            VALUES (?, ?, ?, ?, NULL)
            ON CONFLICT(collection, file_path) DO UPDATE SET
                content_hash = excluded.content_hash,
                indexed_at = excluded.indexed_at,
                deleted_at = NULL
            "#,
        )
        .bind(collection)
        .bind(file_path)
        .bind(hash)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to upsert hash: {e}")))?;

        Ok(())
    }

    /// Mark a file as deleted (tombstone)
    pub async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()> {
        let now = Self::now();

        sqlx::query("UPDATE file_hashes SET deleted_at = ? WHERE collection = ? AND file_path = ?")
            .bind(now)
            .bind(collection)
            .bind(file_path)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Failed to mark deleted: {e}")))?;

        Ok(())
    }

    /// Get all active file paths for a collection
    pub async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT file_path FROM file_hashes WHERE collection = ? AND deleted_at IS NULL",
        )
        .bind(collection)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to get indexed files: {e}")))?;

        Ok(rows.into_iter().map(|(path,)| path).collect())
    }

    /// Cleanup tombstones older than TTL
    /// Returns number of records deleted
    pub async fn cleanup_tombstones(&self) -> Result<u64> {
        let cutoff = Self::now() - self.config.tombstone_ttl_seconds;

        let result =
            sqlx::query("DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
                .bind(cutoff)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database(format!("Failed to cleanup tombstones: {e}")))?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            tracing::info!(deleted = deleted, "Tombstone cleanup complete");
        }

        Ok(deleted)
    }

    /// Cleanup tombstones with custom TTL
    pub async fn cleanup_tombstones_with_ttl(&self, ttl: std::time::Duration) -> Result<u64> {
        let cutoff = Self::now() - ttl.as_secs() as i64;

        let result =
            sqlx::query("DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
                .bind(cutoff)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database(format!("Failed to cleanup tombstones: {e}")))?;

        Ok(result.rows_affected())
    }

    /// Get tombstone count for a collection
    pub async fn tombstone_count(&self, collection: &str) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM file_hashes WHERE collection = ? AND deleted_at IS NOT NULL",
        )
        .bind(collection)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to count tombstones: {e}")))?;

        Ok(result.0)
    }

    /// Clear all records for a collection
    pub async fn clear_collection(&self, collection: &str) -> Result<u64> {
        let result = sqlx::query("DELETE FROM file_hashes WHERE collection = ?")
            .bind(collection)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Failed to clear collection: {e}")))?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_upsert_and_get() {
        let store = FileHashStore::in_memory().await.unwrap();

        // Insert new
        store
            .upsert_hash("test-col", "src/main.rs", "abc123")
            .await
            .unwrap();

        let hash = store.get_hash("test-col", "src/main.rs").await.unwrap();
        assert_eq!(hash, Some("abc123".to_string()));
    }

    #[tokio::test]
    async fn test_has_changed() {
        let store = FileHashStore::in_memory().await.unwrap();

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
        let store = FileHashStore::in_memory().await.unwrap();

        store.upsert_hash("test", "file.rs", "hash").await.unwrap();
        store.mark_deleted("test", "file.rs").await.unwrap();

        // Should not be found after tombstone
        let hash = store.get_hash("test", "file.rs").await.unwrap();
        assert!(hash.is_none());
    }

    #[tokio::test]
    async fn test_resurrect_after_tombstone() {
        let store = FileHashStore::in_memory().await.unwrap();

        store.upsert_hash("test", "file.rs", "hash1").await.unwrap();
        store.mark_deleted("test", "file.rs").await.unwrap();

        // Upsert clears tombstone
        store.upsert_hash("test", "file.rs", "hash2").await.unwrap();

        let hash = store.get_hash("test", "file.rs").await.unwrap();
        assert_eq!(hash, Some("hash2".to_string()));
    }

    #[tokio::test]
    async fn test_get_indexed_files() {
        let store = FileHashStore::in_memory().await.unwrap();

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

    #[test]
    fn test_compute_file_hash() {
        let mut temp = NamedTempFile::new().unwrap();
        use std::io::Write;
        write!(temp, "Hello, World!").unwrap();

        let hash = FileHashStore::compute_file_hash(temp.path()).unwrap();
        // SHA-256 of "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }
}
