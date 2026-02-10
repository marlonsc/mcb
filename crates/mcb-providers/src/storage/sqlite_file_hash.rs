//! SQLite implementation of FileHashRepository.

use async_trait::async_trait;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::repositories::FileHashRepository;
use sha2::{Digest, Sha256};
use sqlx::sqlite::SqlitePool;

/// Configuration for SqliteFileHashRepository
#[derive(Debug, Clone)]
pub struct SqliteFileHashConfig {
    /// Tombstone TTL in seconds (default: 30 days = 2592000)
    pub tombstone_ttl_seconds: i64,
}

impl Default for SqliteFileHashConfig {
    fn default() -> Self {
        Self {
            tombstone_ttl_seconds: 30 * 24 * 60 * 60, // 30 days
        }
    }
}

/// SQLite implementation of file hash tracking
pub struct SqliteFileHashRepository {
    pool: SqlitePool,
    config: SqliteFileHashConfig,
    project_id: String,
}

use crate::database::sqlite::executor::SqliteExecutor;

impl SqliteFileHashRepository {
    /// Create a new SqliteFileHashRepository
    pub fn new(executor: Arc<dyn DatabaseExecutor>, config: SqliteFileHashConfig) -> Self {
        // Downcast executor to SqliteExecutor to get the pool
        let executor = executor
            .as_any()
            .downcast_ref::<SqliteExecutor>()
            .expect("SqliteFileHashRepository requires a SqliteExecutor");

        let pool = executor.pool().clone();

        let project_id = std::env::var("MCB_PROJECT_ID")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            })
            .unwrap_or_else(|| "default".to_string());

        Self {
            pool,
            config,
            project_id,
        }
    }

    /// Get current Unix timestamp
    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    async fn ensure_project_exists(&self) -> Result<()> {
        let now = Self::now();
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO projects (id, name, path, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.project_id)
        .bind(&self.project_id)
        .bind(&self.project_id)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to ensure project exists: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl FileHashRepository for SqliteFileHashRepository {
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT content_hash FROM file_hashes WHERE project_id = ? AND collection = ? AND file_path = ? AND deleted_at IS NULL",
        )
        .bind(&self.project_id)
        .bind(collection)
        .bind(file_path)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to get hash: {e}")))?;

        Ok(result.map(|(hash,)| hash))
    }

    async fn has_changed(
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

    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()> {
        self.ensure_project_exists().await?;
        let now = Self::now();

        sqlx::query(
            r#"
            INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at, deleted_at)
            VALUES (?, ?, ?, ?, ?, NULL)
            ON CONFLICT(project_id, collection, file_path) DO UPDATE SET
                content_hash = excluded.content_hash,
                indexed_at = excluded.indexed_at,
                deleted_at = NULL
            "#,
        )
        .bind(&self.project_id)
        .bind(collection)
        .bind(file_path)
        .bind(hash)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to upsert hash: {e}")))?;

        Ok(())
    }

    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()> {
        let now = Self::now();

        sqlx::query(
            "UPDATE file_hashes SET deleted_at = ? WHERE project_id = ? AND collection = ? AND file_path = ?",
        )
            .bind(now)
            .bind(&self.project_id)
            .bind(collection)
            .bind(file_path)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Failed to mark deleted: {e}")))?;

        Ok(())
    }

    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT file_path FROM file_hashes WHERE project_id = ? AND collection = ? AND deleted_at IS NULL",
        )
        .bind(&self.project_id)
        .bind(collection)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to get indexed files: {e}")))?;

        Ok(rows.into_iter().map(|(path,)| path).collect())
    }

    async fn cleanup_tombstones(&self) -> Result<u64> {
        let cutoff = Self::now() - self.config.tombstone_ttl_seconds;

        let result =
            sqlx::query("DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
                .bind(cutoff)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database(format!("Failed to cleanup tombstones: {e}")))?;

        Ok(result.rows_affected())
    }

    async fn cleanup_tombstones_with_ttl(&self, ttl: std::time::Duration) -> Result<u64> {
        let cutoff = Self::now() - ttl.as_secs() as i64;

        let result =
            sqlx::query("DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
                .bind(cutoff)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::database(format!("Failed to cleanup tombstones: {e}")))?;

        Ok(result.rows_affected())
    }

    async fn tombstone_count(&self, collection: &str) -> Result<i64> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM file_hashes WHERE project_id = ? AND collection = ? AND deleted_at IS NOT NULL",
        )
        .bind(&self.project_id)
        .bind(collection)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::database(format!("Failed to count tombstones: {e}")))?;

        Ok(result.0)
    }

    async fn clear_collection(&self, collection: &str) -> Result<u64> {
        let result = sqlx::query("DELETE FROM file_hashes WHERE project_id = ? AND collection = ?")
            .bind(&self.project_id)
            .bind(collection)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database(format!("Failed to clear collection: {e}")))?;

        Ok(result.rows_affected())
    }

    fn compute_hash(&self, path: &Path) -> Result<String> {
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
}
