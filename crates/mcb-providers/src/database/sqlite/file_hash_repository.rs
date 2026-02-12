//! SQLite file-hash repository using the domain port [`DatabaseExecutor`].
//!
//! Implements [`FileHashRepository`] via [`DatabaseExecutor`]; no direct sqlx
//! dependency — all SQL goes through the portable `execute` / `query_one` /
//! `query_all` abstraction, matching the pattern of every other repository
//! in this module (agent, project, memory, …).

use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::FileHashRepository;
use sha2::{Digest, Sha256};

/// Configuration for [`SqliteFileHashRepository`].
#[derive(Debug, Clone)]
pub struct SqliteFileHashConfig {
    /// Tombstone TTL in seconds (default: 30 days = 2_592_000).
    pub tombstone_ttl_seconds: i64,
}

impl Default for SqliteFileHashConfig {
    fn default() -> Self {
        Self {
            tombstone_ttl_seconds: 30 * 24 * 60 * 60, // 30 days
        }
    }
}

/// SQLite-based file-hash repository using the database executor port.
pub struct SqliteFileHashRepository {
    executor: Arc<dyn DatabaseExecutor>,
    config: SqliteFileHashConfig,
}

impl SqliteFileHashRepository {
    /// Create a new repository backed by the given executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>, config: SqliteFileHashConfig) -> Self {
        Self { executor, config }
    }

    /// Current Unix timestamp (seconds).
    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }
}

#[async_trait]
impl FileHashRepository for SqliteFileHashRepository {
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>> {
        let row = self
            .executor
            .query_one(
                "SELECT content_hash FROM file_hashes WHERE collection = ? AND file_path = ? AND deleted_at IS NULL",
                &[
                    SqlParam::String(collection.to_string()),
                    SqlParam::String(file_path.to_string()),
                ],
            )
            .await?;

        match row {
            Some(r) => Ok(r.try_get_string("content_hash")?),
            None => Ok(None),
        }
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
        let now = Self::now();

        self.executor
            .execute(
                r"
                INSERT INTO file_hashes (collection, file_path, content_hash, indexed_at, deleted_at)
                VALUES (?, ?, ?, ?, NULL)
                ON CONFLICT(collection, file_path) DO UPDATE SET
                    content_hash = excluded.content_hash,
                    indexed_at = excluded.indexed_at,
                    deleted_at = NULL
                ",
                &[
                    SqlParam::String(collection.to_string()),
                    SqlParam::String(file_path.to_string()),
                    SqlParam::String(hash.to_string()),
                    SqlParam::I64(now),
                ],
            )
            .await
    }

    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()> {
        let now = Self::now();

        self.executor
            .execute(
                "UPDATE file_hashes SET deleted_at = ? WHERE collection = ? AND file_path = ?",
                &[
                    SqlParam::I64(now),
                    SqlParam::String(collection.to_string()),
                    SqlParam::String(file_path.to_string()),
                ],
            )
            .await
    }

    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>> {
        let rows = self
            .executor
            .query_all(
                "SELECT file_path FROM file_hashes WHERE collection = ? AND deleted_at IS NULL",
                &[SqlParam::String(collection.to_string())],
            )
            .await?;

        let mut paths = Vec::with_capacity(rows.len());
        for row in rows {
            if let Some(path) = row.try_get_string("file_path")? {
                paths.push(path);
            }
        }
        Ok(paths)
    }

    async fn cleanup_tombstones(&self) -> Result<u64> {
        let cutoff = Self::now() - self.config.tombstone_ttl_seconds;

        let rows = self
            .executor
            .query_all(
                "SELECT id FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                &[SqlParam::I64(cutoff)],
            )
            .await?;
        let count = rows.len() as u64;

        if count > 0 {
            self.executor
                .execute(
                    "DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                    &[SqlParam::I64(cutoff)],
                )
                .await?;
        }

        Ok(count)
    }

    async fn cleanup_tombstones_with_ttl(&self, ttl: Duration) -> Result<u64> {
        let cutoff = Self::now() - ttl.as_secs() as i64;

        let rows = self
            .executor
            .query_all(
                "SELECT id FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                &[SqlParam::I64(cutoff)],
            )
            .await?;
        let count = rows.len() as u64;

        if count > 0 {
            self.executor
                .execute(
                    "DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                    &[SqlParam::I64(cutoff)],
                )
                .await?;
        }

        Ok(count)
    }

    async fn tombstone_count(&self, collection: &str) -> Result<i64> {
        let row = self
            .executor
            .query_one(
                "SELECT COUNT(*) as cnt FROM file_hashes WHERE collection = ? AND deleted_at IS NOT NULL",
                &[SqlParam::String(collection.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(r.try_get_i64("cnt")?.unwrap_or(0)),
            None => Ok(0),
        }
    }

    async fn clear_collection(&self, collection: &str) -> Result<u64> {
        let rows = self
            .executor
            .query_all(
                "SELECT id FROM file_hashes WHERE collection = ?",
                &[SqlParam::String(collection.to_string())],
            )
            .await?;
        let count = rows.len() as u64;

        if count > 0 {
            self.executor
                .execute(
                    "DELETE FROM file_hashes WHERE collection = ?",
                    &[SqlParam::String(collection.to_string())],
                )
                .await?;
        }

        Ok(count)
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
