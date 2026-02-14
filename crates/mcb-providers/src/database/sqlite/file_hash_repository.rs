//! SQLite File Hash Repository
//!
//! # Overview
//! The `SqliteFileHashRepository` tracks file content hashes to optimize the indexing process.
//! It allows the system to skip re-indexing files that haven't changed, significantly improving
//! performance for large codebases.
//!
//! # Responsibilities
//! - **Change Detection**: Comparing current file hashes against stored values.
//! - **Tombstone Management**: Handling soft-deletion of files with a configurable TTL.
//! - **Project Isolation**: Scoping all hash tracking to specific project IDs.
//! - **Integrity Checks**: Computing SHA-256 hashes of file contents on disk.

use async_trait::async_trait;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::infrastructure::database::SqlParam;
use mcb_domain::ports::repositories::FileHashRepository;
use serde_json::json;
use sha2::{Digest, Sha256};

/// Configuration for SqliteFileHashRepository
#[derive(Debug, Clone)]
pub struct SqliteFileHashConfig {
    /// Tombstone TTL in seconds (default: 30 days = 2592000)
    pub tombstone_ttl_seconds: i64,
}

impl Default for SqliteFileHashConfig {
    fn default() -> Self {
        Self {
            tombstone_ttl_seconds: 30 * 24 * 60 * 60,
        }
    }
}

/// SQLite-based implementation of `FileHashRepository`.
///
/// Manages the `file_hashes` and `collections` tables. It provides mechanism
/// to upsert hashes, query changes, and perform garbage collection on deleted file records (tombstones).
pub struct SqliteFileHashRepository {
    executor: Arc<dyn DatabaseExecutor>,
    config: SqliteFileHashConfig,
    project_id: String,
}

impl SqliteFileHashRepository {
    /// Create a new SqliteFileHashRepository
    pub fn new(executor: Arc<dyn DatabaseExecutor>, config: SqliteFileHashConfig) -> Self {
        // TODO(architecture): Inject project_id explicitly instead of inferring from env/CWD.
        // Repositories should not depend on ambient environment variables for core identity.
        let project_id = std::env::vars()
            .find_map(|(key, value)| {
                if key == "MCB_PROJECT_ID" {
                    Some(value)
                } else {
                    None
                }
            })
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
            })
            .unwrap_or_else(|| "default".to_string());

        Self {
            executor,
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
        crate::database::sqlite::ensure_parent::ensure_org_and_project(
            self.executor.as_ref(),
            &self.project_id,
            now,
        )
        .await
    }

    async fn query_opt_string(
        &self,
        sql: &str,
        params: &[SqlParam],
        column: &str,
    ) -> Result<Option<String>> {
        let row = self.executor.query_one(sql, params).await?;
        match row {
            Some(r) => r.try_get_string(column),
            None => Ok(None),
        }
    }

    async fn query_strings(
        &self,
        sql: &str,
        params: &[SqlParam],
        column: &str,
    ) -> Result<Vec<String>> {
        let rows = self.executor.query_all(sql, params).await?;
        rows.into_iter()
            .map(|row| row.try_get_string(column))
            .filter_map(|value| match value {
                Ok(Some(text)) => Some(Ok(text)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            })
            .collect()
    }

    async fn query_count(&self, sql: &str, params: &[SqlParam]) -> Result<u64> {
        let row = self.executor.query_one(sql, params).await?;
        let count = row
            .and_then(|r| r.try_get_i64("count").ok().flatten())
            .unwrap_or(0);
        Ok(count.max(0) as u64)
    }
}

#[async_trait]
impl FileHashRepository for SqliteFileHashRepository {
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>> {
        self.query_opt_string(
            "SELECT content_hash FROM file_hashes WHERE project_id = ? AND collection = ? AND file_path = ? AND deleted_at IS NULL",
            &[
                SqlParam::String(self.project_id.clone()),
                SqlParam::String(collection.to_string()),
                SqlParam::String(file_path.to_string()),
            ],
            "content_hash",
        )
        .await
    }

    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool> {
        match self.get_hash(collection, file_path).await? {
            Some(stored_hash) => Ok(stored_hash != current_hash),
            None => Ok(true),
        }
    }

    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()> {
        self.ensure_project_exists().await?;
        let now = Self::now();

        self.executor
            .execute(
                r#"
            INSERT OR IGNORE INTO collections (id, project_id, name, vector_name, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
                &[
                    SqlParam::String(format!("{}:{}", self.project_id, collection)),
                    SqlParam::String(self.project_id.clone()),
                    SqlParam::String(collection.to_string()),
                    SqlParam::String(collection.to_string()),
                    SqlParam::I64(now),
                ],
            )
            .await
            .map_err(|e| Error::database(format!("Failed to ensure collection metadata: {e}")))?;

        self.executor
            .execute(
                r#"
            INSERT INTO file_hashes (project_id, collection, file_path, content_hash, indexed_at, deleted_at, origin_context)
            VALUES (?, ?, ?, ?, ?, NULL, ?)
            ON CONFLICT(project_id, collection, file_path) DO UPDATE SET
                content_hash = excluded.content_hash,
                indexed_at = excluded.indexed_at,
                deleted_at = NULL,
                origin_context = excluded.origin_context
            "#,
                &[
                    SqlParam::String(self.project_id.clone()),
                    SqlParam::String(collection.to_string()),
                    SqlParam::String(file_path.to_string()),
                    SqlParam::String(hash.to_string()),
                    SqlParam::I64(now),
                    SqlParam::String(
                        json!({
                            "project_id": self.project_id.clone(),
                            "collection": collection,
                            "file_path": file_path,
                            "timestamp": now,
                        })
                        .to_string(),
                    ),
                ],
            )
            .await
            .map_err(|e| Error::database(format!("Failed to upsert hash: {e}")))?;

        Ok(())
    }

    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()> {
        let now = Self::now();

        self.executor
            .execute(
                "UPDATE file_hashes SET deleted_at = ? WHERE project_id = ? AND collection = ? AND file_path = ?",
                &[
                    SqlParam::I64(now),
                    SqlParam::String(self.project_id.clone()),
                    SqlParam::String(collection.to_string()),
                    SqlParam::String(file_path.to_string()),
                ],
            )
            .await
            .map_err(|e| Error::database(format!("Failed to mark deleted: {e}")))?;

        Ok(())
    }

    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>> {
        self.query_strings(
            "SELECT file_path FROM file_hashes WHERE project_id = ? AND collection = ? AND deleted_at IS NULL",
            &[
                SqlParam::String(self.project_id.clone()),
                SqlParam::String(collection.to_string()),
            ],
            "file_path",
        )
        .await
        .map_err(|e| Error::database(format!("Failed to get indexed files: {e}")))
    }

    async fn cleanup_tombstones(&self) -> Result<u64> {
        let cutoff = Self::now() - self.config.tombstone_ttl_seconds;

        let delete_params = &[SqlParam::I64(cutoff)];
        let count = self
            .query_count(
                "SELECT COUNT(*) as count FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                delete_params,
            )
            .await?;
        self.executor
            .execute(
                "DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                delete_params,
            )
            .await
            .map_err(|e| Error::database(format!("Failed to cleanup tombstones: {e}")))?;
        Ok(count)
    }

    async fn cleanup_tombstones_with_ttl(&self, ttl: std::time::Duration) -> Result<u64> {
        let cutoff = Self::now() - ttl.as_secs() as i64;

        let delete_params = &[SqlParam::I64(cutoff)];
        let count = self
            .query_count(
                "SELECT COUNT(*) as count FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                delete_params,
            )
            .await?;
        self.executor
            .execute(
                "DELETE FROM file_hashes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
                delete_params,
            )
            .await
            .map_err(|e| Error::database(format!("Failed to cleanup tombstones: {e}")))?;
        Ok(count)
    }

    async fn tombstone_count(&self, collection: &str) -> Result<i64> {
        let result = self
            .query_count(
                "SELECT COUNT(*) as count FROM file_hashes WHERE project_id = ? AND collection = ? AND deleted_at IS NOT NULL",
                &[
                    SqlParam::String(self.project_id.clone()),
                    SqlParam::String(collection.to_string()),
                ],
            )
            .await
            .map_err(|e| Error::database(format!("Failed to count tombstones: {e}")))?;

        Ok(result as i64)
    }

    async fn clear_collection(&self, collection: &str) -> Result<u64> {
        let params = &[
            SqlParam::String(self.project_id.clone()),
            SqlParam::String(collection.to_string()),
        ];
        let count = self
            .query_count(
                "SELECT COUNT(*) as count FROM file_hashes WHERE project_id = ? AND collection = ?",
                params,
            )
            .await?;
        self.executor
            .execute(
                "DELETE FROM file_hashes WHERE project_id = ? AND collection = ?",
                params,
            )
            .await
            .map_err(|e| Error::database(format!("Failed to clear collection: {e}")))?;

        Ok(count)
    }

    // TODO(architecture): Extract hash computation to a separate service or utility port.
    // Mixing file I/O with database logic violates the Single Responsibility Principle and complicates testing.
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
