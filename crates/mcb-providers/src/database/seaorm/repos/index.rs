//! SeaORM-based Index Repository
//!
//! Persists indexing operation state in the database using the `index_operations`,
//! `collections`, and `file_hashes` tables. Provides durable progress tracking
//! that survives process restarts.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    FileHashRepository, IndexRepository, IndexStats, IndexingOperation, IndexingOperationStatus,
};
use mcb_domain::value_objects::{CollectionId, OperationId};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::database::seaorm::entities::{collection, file_hash, index_operation};

/// `SeaORM` implementation of `IndexRepository`.
///
/// Uses three tables:
/// - `index_operations`: Tracks active/completed indexing operations
/// - `collections`: Collection metadata
/// - `file_hashes`: Per-file indexing state
pub struct SeaOrmIndexRepository {
    db: Arc<DatabaseConnection>,
    project_id: String,
}

impl SeaOrmIndexRepository {
    /// Create a new `SeaOrmIndexRepository`.
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>, project_id: String) -> Self {
        Self { db, project_id }
    }

    fn db_err<E>(context: &str, source: E) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Database {
            message: context.to_owned(),
            source: Some(Box::new(source)),
        }
    }

    fn now() -> Result<i64> {
        mcb_domain::utils::time::epoch_secs_i64()
    }

    fn status_to_string(status: &IndexingOperationStatus) -> String {
        match status {
            IndexingOperationStatus::Starting => "starting".to_owned(),
            IndexingOperationStatus::InProgress => "in_progress".to_owned(),
            IndexingOperationStatus::Completed => "completed".to_owned(),
            IndexingOperationStatus::Failed(msg) => format!("failed:{msg}"),
        }
    }

    fn string_to_status(s: &str) -> IndexingOperationStatus {
        match s {
            "starting" => IndexingOperationStatus::Starting,
            "in_progress" => IndexingOperationStatus::InProgress,
            "completed" => IndexingOperationStatus::Completed,
            other if other.starts_with("failed:") => IndexingOperationStatus::Failed(
                other
                    .strip_prefix("failed:")
                    .unwrap_or("error message missing from database")
                    .to_owned(),
            ),
            _ => IndexingOperationStatus::Failed(format!("unknown status: {s}")),
        }
    }

    fn model_to_domain(model: index_operation::Model) -> IndexingOperation {
        IndexingOperation {
            id: OperationId::from_string(&model.id),
            collection: CollectionId::from_string(&model.collection_id),
            status: Self::string_to_status(&model.status),
            total_files: model.total_files as usize,
            processed_files: model.processed_files as usize,
            current_file: model.current_file,
            started_at: model.started_at,
        }
    }
}

#[async_trait]
impl IndexRepository for SeaOrmIndexRepository {
    async fn start_indexing(
        &self,
        collection_id: &CollectionId,
        total_files: usize,
    ) -> Result<OperationId> {
        let now = Self::now()?;
        let op_id = OperationId::new();

        let model = index_operation::ActiveModel {
            id: Set(op_id.as_str()),
            collection_id: Set(collection_id.as_str()),
            status: Set(Self::status_to_string(&IndexingOperationStatus::Starting)),
            total_files: Set(total_files as i64),
            processed_files: Set(0),
            current_file: Set(None),
            error_message: Set(None),
            started_at: Set(now),
            completed_at: Set(None),
        };

        index_operation::Entity::insert(model)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("start indexing operation", e))?;

        Ok(op_id)
    }

    async fn get_operation(&self, operation_id: &OperationId) -> Result<Option<IndexingOperation>> {
        let result = index_operation::Entity::find_by_id(operation_id.as_str())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get indexing operation", e))?;

        Ok(result.map(Self::model_to_domain))
    }

    async fn list_operations(&self) -> Result<Vec<IndexingOperation>> {
        let results = index_operation::Entity::find()
            .order_by_desc(index_operation::Column::StartedAt)
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("list indexing operations", e))?;

        Ok(results.into_iter().map(Self::model_to_domain).collect())
    }

    async fn get_active_operation(
        &self,
        collection_id: &CollectionId,
    ) -> Result<Option<IndexingOperation>> {
        let result = index_operation::Entity::find()
            .filter(index_operation::Column::CollectionId.eq(collection_id.as_str()))
            .filter(index_operation::Column::Status.is_in(["starting", "in_progress"]))
            .order_by_desc(index_operation::Column::StartedAt)
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get active indexing operation", e))?;

        Ok(result.map(Self::model_to_domain))
    }

    async fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed_files: usize,
    ) -> Result<()> {
        let existing = index_operation::Entity::find_by_id(operation_id.as_str())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("find operation for progress update", e))?
            .ok_or_else(|| Error::NotFound {
                resource: format!("IndexOperation:{operation_id}"),
            })?;

        let mut active: index_operation::ActiveModel = existing.into();
        active.status = Set(Self::status_to_string(&IndexingOperationStatus::InProgress));
        active.processed_files = Set(processed_files as i64);
        active.current_file = Set(current_file);

        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("update indexing progress", e))?;

        Ok(())
    }

    async fn complete_operation(&self, operation_id: &OperationId) -> Result<()> {
        let now = Self::now()?;

        let existing = index_operation::Entity::find_by_id(operation_id.as_str())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("find operation for completion", e))?
            .ok_or_else(|| Error::NotFound {
                resource: format!("IndexOperation:{operation_id}"),
            })?;

        let mut active: index_operation::ActiveModel = existing.into();
        active.status = Set(Self::status_to_string(&IndexingOperationStatus::Completed));
        active.completed_at = Set(Some(now));

        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("complete indexing operation", e))?;

        Ok(())
    }

    async fn fail_operation(&self, operation_id: &OperationId, error: &str) -> Result<()> {
        let now = Self::now()?;

        let existing = index_operation::Entity::find_by_id(operation_id.as_str())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("find operation for failure", e))?
            .ok_or_else(|| Error::NotFound {
                resource: format!("IndexOperation:{operation_id}"),
            })?;

        let mut active: index_operation::ActiveModel = existing.into();
        active.status = Set(Self::status_to_string(&IndexingOperationStatus::Failed(
            error.to_owned(),
        )));
        active.error_message = Set(Some(error.to_owned()));
        active.completed_at = Set(Some(now));

        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("fail indexing operation", e))?;

        Ok(())
    }

    async fn clear_index(&self, collection_id: &CollectionId) -> Result<u64> {
        let collection_str = collection_id.as_str();

        // Count file hashes to report
        let file_hashes = file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(&collection_str))
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("count file hashes for clear", e))?;

        let count = file_hashes.len() as u64;

        // Delete file hashes for this collection
        file_hash::Entity::delete_many()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(&collection_str))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("clear file hashes", e))?;

        // Delete collection metadata
        let collection_id_str = format!("{}:{}", self.project_id, collection_str);
        collection::Entity::delete_by_id(&collection_id_str)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("clear collection metadata", e))?;

        // Mark any active operations for this collection as failed
        let active_ops = index_operation::Entity::find()
            .filter(index_operation::Column::CollectionId.eq(&collection_str))
            .filter(index_operation::Column::Status.is_in(["starting", "in_progress"]))
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("find active ops for clear", e))?;

        let now = Self::now()?;
        for op in active_ops {
            let mut active: index_operation::ActiveModel = op.into();
            active.status = Set(Self::status_to_string(&IndexingOperationStatus::Failed(
                "index cleared".to_owned(),
            )));
            active.completed_at = Set(Some(now));
            active
                .update(self.db.as_ref())
                .await
                .map_err(|e| Self::db_err("cancel active op during clear", e))?;
        }

        Ok(count)
    }

    async fn get_index_stats(&self, collection_id: &CollectionId) -> Result<IndexStats> {
        let collection_str = collection_id.as_str();

        let indexed_files = file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(&collection_str))
            .filter(file_hash::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("count indexed files", e))?;

        let last_op = index_operation::Entity::find()
            .filter(index_operation::Column::CollectionId.eq(&collection_str))
            .filter(index_operation::Column::Status.eq("completed"))
            .order_by_desc(index_operation::Column::CompletedAt)
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get last completed operation", e))?;

        let active_op = index_operation::Entity::find()
            .filter(index_operation::Column::CollectionId.eq(&collection_str))
            .filter(index_operation::Column::Status.is_in(["starting", "in_progress"]))
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("check active indexing", e))?;

        Ok(IndexStats {
            indexed_files: indexed_files.len() as u64,
            last_indexed_at: last_op.and_then(|op| op.completed_at),
            is_indexing: active_op.is_some(),
        })
    }
}

#[async_trait]
impl FileHashRepository for SeaOrmIndexRepository {
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>> {
        let result = file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
            .filter(file_hash::Column::FilePath.eq(file_path))
            .filter(file_hash::Column::DeletedAt.is_null())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get file hash", e))?;

        Ok(result.map(|m| m.content_hash))
    }

    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool> {
        match self.get_hash(collection, file_path).await? {
            Some(stored) => Ok(stored != current_hash),
            None => Ok(true),
        }
    }

    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()> {
        use sea_orm::TransactionTrait;

        let now = Self::now()?;
        let project_id = self.project_id.clone();
        let collection = collection.to_owned();
        let file_path = file_path.to_owned();
        let hash = hash.to_owned();

        self.db
            .as_ref()
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let existing = file_hash::Entity::find()
                        .filter(file_hash::Column::ProjectId.eq(&project_id))
                        .filter(file_hash::Column::Collection.eq(&collection))
                        .filter(file_hash::Column::FilePath.eq(&file_path))
                        .one(txn)
                        .await?;

                    if let Some(model) = existing {
                        let mut active: file_hash::ActiveModel = model.into();
                        active.content_hash = Set(hash);
                        active.indexed_at = Set(now);
                        active.deleted_at = Set(None);
                        active.update(txn).await?;
                    } else {
                        let active = file_hash::ActiveModel {
                            id: sea_orm::ActiveValue::NotSet,
                            project_id: Set(project_id),
                            collection: Set(collection),
                            file_path: Set(file_path),
                            content_hash: Set(hash),
                            indexed_at: Set(now),
                            deleted_at: Set(None),
                            origin_context: Set(None),
                        };
                        file_hash::Entity::insert(active).exec(txn).await?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| Self::db_err("upsert file hash", e))
    }

    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()> {
        let now = Self::now()?;

        let existing = file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
            .filter(file_hash::Column::FilePath.eq(file_path))
            .filter(file_hash::Column::DeletedAt.is_null())
            .one(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("find file hash for tombstone", e))?;

        if let Some(model) = existing {
            let mut active: file_hash::ActiveModel = model.into();
            active.deleted_at = Set(Some(now));
            active
                .update(self.db.as_ref())
                .await
                .map_err(|e| Self::db_err("mark file hash deleted", e))?;
        }

        Ok(())
    }

    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>> {
        let results = file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
            .filter(file_hash::Column::DeletedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("get indexed files", e))?;

        Ok(results.into_iter().map(|m| m.file_path).collect())
    }

    async fn cleanup_tombstones(&self) -> Result<u64> {
        self.cleanup_tombstones_with_ttl(Duration::from_secs(7 * 24 * 3600))
            .await
    }

    async fn cleanup_tombstones_with_ttl(&self, ttl: Duration) -> Result<u64> {
        let now = Self::now()?;
        let cutoff = now - ttl.as_secs() as i64;

        let result = file_hash::Entity::delete_many()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::DeletedAt.is_not_null())
            .filter(file_hash::Column::DeletedAt.lt(cutoff))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("cleanup tombstones", e))?;

        Ok(result.rows_affected)
    }

    async fn tombstone_count(&self, collection: &str) -> Result<i64> {
        let results = file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
            .filter(file_hash::Column::DeletedAt.is_not_null())
            .all(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("count tombstones", e))?;

        Ok(results.len() as i64)
    }

    async fn clear_collection(&self, collection: &str) -> Result<u64> {
        let result = file_hash::Entity::delete_many()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| Self::db_err("clear collection file hashes", e))?;

        Ok(result.rows_affected)
    }

    fn compute_hash(&self, path: &std::path::Path) -> Result<String> {
        use std::io::{BufReader, Read};

        use sha2::{Digest, Sha256};

        let file = std::fs::File::open(path).map_err(|e| Error::Database {
            message: format!("open file for hashing: {}", path.display()),
            source: Some(Box::new(e)),
        })?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = reader.read(&mut buf).map_err(|e| Error::Database {
                message: format!("read file for hashing: {}", path.display()),
                source: Some(Box::new(e)),
            })?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}
