//! SeaORM-based Index Repository
//!
//! Persists indexing state using `index_operations`, `collections`, and
//! `file_hashes` tables. Provides durable progress tracking across restarts.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    FileHashRepository, IndexRepository, IndexStats, IndexingOperation, IndexingOperationStatus,
};
use mcb_domain::value_objects::{CollectionId, OperationId};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use super::common::db_error;
use crate::database::seaorm::entities::{collection, file_hash, index_operation};

/// SeaORM `IndexRepository` + `FileHashRepository` implementation.
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

    /// Returns the current epoch time in seconds.
    fn now() -> Result<i64> {
        Ok(mcb_utils::utils::time::epoch_secs_i64()?)
    }

    fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }

    /// Returns a `Select` pre-filtered by `(project_id, collection)`.
    fn file_hash_query(&self, collection: &str) -> sea_orm::Select<file_hash::Entity> {
        file_hash::Entity::find()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
    }

    fn status_to_string(status: &IndexingOperationStatus) -> String {
        match status {
            IndexingOperationStatus::Starting => {
                mcb_utils::constants::INDEX_OP_STATUS_STARTING.to_owned()
            }
            IndexingOperationStatus::InProgress => {
                mcb_utils::constants::INDEX_OP_STATUS_IN_PROGRESS.to_owned()
            }
            IndexingOperationStatus::Completed => {
                mcb_utils::constants::INDEX_OP_STATUS_COMPLETED.to_owned()
            }
            IndexingOperationStatus::Failed(msg) => {
                format!("{}:{msg}", mcb_utils::constants::INDEX_OP_STATUS_FAILED)
            }
        }
    }

    fn string_to_status(s: &str) -> IndexingOperationStatus {
        match s {
            s if s == mcb_utils::constants::INDEX_OP_STATUS_STARTING => {
                IndexingOperationStatus::Starting
            }
            s if s == mcb_utils::constants::INDEX_OP_STATUS_IN_PROGRESS => {
                IndexingOperationStatus::InProgress
            }
            s if s == mcb_utils::constants::INDEX_OP_STATUS_COMPLETED => {
                IndexingOperationStatus::Completed
            }
            other if other.starts_with(mcb_utils::constants::INDEX_OP_STATUS_FAILED) => {
                IndexingOperationStatus::Failed(
                    other
                        .strip_prefix(&format!(
                            "{}:",
                            mcb_utils::constants::INDEX_OP_STATUS_FAILED
                        ))
                        .unwrap_or("error message missing from database")
                        .to_owned(),
                )
            }
            _ => IndexingOperationStatus::Failed(format!("unknown status: {s}")),
        }
    }

    /// Helper: filter for active (starting | in-progress) operations on a collection.
    fn active_ops_query(collection_str: &str) -> sea_orm::Select<index_operation::Entity> {
        index_operation::Entity::find()
            .filter(index_operation::Column::CollectionId.eq(collection_str))
            .filter(index_operation::Column::Status.is_in([
                mcb_utils::constants::INDEX_OP_STATUS_STARTING,
                mcb_utils::constants::INDEX_OP_STATUS_IN_PROGRESS,
            ]))
    }

    /// Helper: find + update an operation by ID with error mapping.
    async fn find_and_update_op(
        &self,
        operation_id: &OperationId,
        ctx: &str,
        mutate: impl FnOnce(&mut index_operation::ActiveModel),
    ) -> Result<()> {
        let existing = index_operation::Entity::find_by_id(operation_id.as_str())
            .one(self.db())
            .await
            .map_err(db_error(&format!("find operation for {ctx}")))?
            .ok_or_else(|| Error::NotFound {
                resource: format!("IndexOperation:{operation_id}"),
            })?;
        let mut active: index_operation::ActiveModel = existing.into();
        mutate(&mut active);
        active.update(self.db()).await.map_err(db_error(ctx))?;
        Ok(())
    }

    async fn fail_active_operations_for_collection(
        &self,
        collection_str: &str,
        now: i64,
    ) -> Result<()> {
        let active_ops = Self::active_ops_query(collection_str)
            .all(self.db())
            .await
            .map_err(db_error("find active ops for clear"))?;
        for op in active_ops {
            let mut active: index_operation::ActiveModel = op.into();
            active.status = Set(Self::status_to_string(&IndexingOperationStatus::Failed(
                "index cleared".to_owned(),
            )));
            active.completed_at = Set(Some(now));
            active
                .update(self.db())
                .await
                .map_err(db_error("cancel active op during clear"))?;
        }
        Ok(())
    }
}

impl From<index_operation::Model> for IndexingOperation {
    fn from(model: index_operation::Model) -> Self {
        Self {
            id: OperationId::from_string(&model.id),
            collection: CollectionId::from_string(&model.collection_id),
            status: SeaOrmIndexRepository::string_to_status(&model.status),
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
            .exec(self.db())
            .await
            .map_err(db_error("start indexing operation"))?;
        Ok(op_id)
    }

    async fn get_operation(&self, operation_id: &OperationId) -> Result<Option<IndexingOperation>> {
        sea_repo_get_opt!(
            self.db(),
            index_operation,
            IndexingOperation,
            operation_id.as_str(),
            "get indexing operation"
        )
    }

    async fn list_operations(&self) -> Result<Vec<IndexingOperation>> {
        let results = index_operation::Entity::find()
            .order_by_desc(index_operation::Column::StartedAt)
            .all(self.db())
            .await
            .map_err(db_error("list indexing operations"))?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_active_operation(
        &self,
        collection_id: &CollectionId,
    ) -> Result<Option<IndexingOperation>> {
        let result = Self::active_ops_query(&collection_id.as_str())
            .order_by_desc(index_operation::Column::StartedAt)
            .one(self.db())
            .await
            .map_err(db_error("get active indexing operation"))?;
        Ok(result.map(Into::into))
    }

    async fn update_progress(
        &self,
        operation_id: &OperationId,
        current_file: Option<String>,
        processed_files: usize,
    ) -> Result<()> {
        self.find_and_update_op(operation_id, "update indexing progress", |op| {
            op.status = Set(Self::status_to_string(&IndexingOperationStatus::InProgress));
            op.processed_files = Set(processed_files as i64);
            op.current_file = Set(current_file);
        })
        .await
    }

    async fn complete_operation(&self, operation_id: &OperationId) -> Result<()> {
        let now = Self::now()?;
        self.find_and_update_op(operation_id, "complete indexing operation", |op| {
            op.status = Set(Self::status_to_string(&IndexingOperationStatus::Completed));
            op.completed_at = Set(Some(now));
        })
        .await
    }

    async fn fail_operation(&self, operation_id: &OperationId, error: &str) -> Result<()> {
        let now = Self::now()?;
        let error_owned = error.to_owned();
        self.find_and_update_op(operation_id, "fail indexing operation", |op| {
            op.status = Set(Self::status_to_string(&IndexingOperationStatus::Failed(
                error_owned.clone(),
            )));
            op.error_message = Set(Some(error_owned));
            op.completed_at = Set(Some(now));
        })
        .await
    }

    async fn clear_index(&self, collection_id: &CollectionId) -> Result<u64> {
        let col = collection_id.as_str();

        let count = self
            .file_hash_query(&col)
            .count(self.db())
            .await
            .map_err(db_error("count file hashes for clear"))?;

        file_hash::Entity::delete_many()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(&col))
            .exec(self.db())
            .await
            .map_err(db_error("clear file hashes"))?;

        let collection_id_str = format!("{}:{}", self.project_id, col);
        collection::Entity::delete_by_id(&collection_id_str)
            .exec(self.db())
            .await
            .map_err(db_error("clear collection metadata"))?;

        self.fail_active_operations_for_collection(&col, Self::now()?)
            .await?;
        Ok(count)
    }

    async fn get_index_stats(&self, collection_id: &CollectionId) -> Result<IndexStats> {
        let col = collection_id.as_str();
        let indexed_files = self
            .file_hash_query(&col)
            .filter(file_hash::Column::DeletedAt.is_null())
            .count(self.db())
            .await
            .map_err(db_error("count indexed files"))?;

        let last_op = index_operation::Entity::find()
            .filter(index_operation::Column::CollectionId.eq(&col))
            .filter(
                index_operation::Column::Status.eq(mcb_utils::constants::INDEX_OP_STATUS_COMPLETED),
            )
            .order_by_desc(index_operation::Column::CompletedAt)
            .one(self.db())
            .await
            .map_err(db_error("get last completed operation"))?;

        let is_indexing = Self::active_ops_query(&col)
            .one(self.db())
            .await
            .map_err(db_error("check active indexing"))?
            .is_some();

        Ok(IndexStats {
            indexed_files,
            last_indexed_at: last_op.and_then(|op| op.completed_at),
            is_indexing,
        })
    }
}

#[async_trait]
impl FileHashRepository for SeaOrmIndexRepository {
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>> {
        let result = self
            .file_hash_query(collection)
            .filter(file_hash::Column::FilePath.eq(file_path))
            .filter(file_hash::Column::DeletedAt.is_null())
            .one(self.db())
            .await
            .map_err(db_error("get file hash"))?;
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

        self.db()
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
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(err)
                | sea_orm::TransactionError::Transaction(err) => db_error("upsert file hash")(err),
            })
    }

    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()> {
        let now = Self::now()?;
        let existing = self
            .file_hash_query(collection)
            .filter(file_hash::Column::FilePath.eq(file_path))
            .filter(file_hash::Column::DeletedAt.is_null())
            .one(self.db())
            .await
            .map_err(db_error("find file hash for tombstone"))?;
        if let Some(model) = existing {
            let mut active: file_hash::ActiveModel = model.into();
            active.deleted_at = Set(Some(now));
            active
                .update(self.db())
                .await
                .map_err(db_error("mark file hash deleted"))?;
        }
        Ok(())
    }

    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>> {
        let results = self
            .file_hash_query(collection)
            .filter(file_hash::Column::DeletedAt.is_null())
            .all(self.db())
            .await
            .map_err(db_error("get indexed files"))?;
        Ok(results.into_iter().map(|m| m.file_path).collect())
    }

    async fn cleanup_tombstones(&self) -> Result<u64> {
        self.cleanup_tombstones_with_ttl(Duration::from_secs(
            mcb_utils::constants::TOMBSTONE_TTL_SECS,
        ))
        .await
    }

    async fn cleanup_tombstones_with_ttl(&self, ttl: Duration) -> Result<u64> {
        let cutoff = Self::now()? - ttl.as_secs() as i64;
        let result = file_hash::Entity::delete_many()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::DeletedAt.is_not_null())
            .filter(file_hash::Column::DeletedAt.lt(cutoff))
            .exec(self.db())
            .await
            .map_err(db_error("cleanup tombstones"))?;
        Ok(result.rows_affected)
    }

    async fn tombstone_count(&self, collection: &str) -> Result<i64> {
        let count = self
            .file_hash_query(collection)
            .filter(file_hash::Column::DeletedAt.is_not_null())
            .count(self.db())
            .await
            .map_err(db_error("count tombstones"))?;
        Ok(count as i64)
    }

    async fn clear_collection(&self, collection: &str) -> Result<u64> {
        let result = file_hash::Entity::delete_many()
            .filter(file_hash::Column::ProjectId.eq(&self.project_id))
            .filter(file_hash::Column::Collection.eq(collection))
            .exec(self.db())
            .await
            .map_err(db_error("clear collection file hashes"))?;
        Ok(result.rows_affected)
    }

    fn compute_hash(&self, path: &std::path::Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::io::{BufReader, Read};

        let file = std::fs::File::open(path).map_err(|e| {
            Error::database_with_source(format!("open file for hashing: {}", path.display()), e)
        })?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = reader.read(&mut buf).map_err(|e| {
                Error::database_with_source(format!("read file for hashing: {}", path.display()), e)
            })?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}
