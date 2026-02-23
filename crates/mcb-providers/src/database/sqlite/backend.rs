//!
//! SQLite backend that combines the port executor with a SeaORM connection.
//!
//! Used by repositories that can use SeaORM for simpler CRUD while still
//! exposing a single `DatabaseExecutor` to the rest of the stack.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::{DatabaseExecutor, SqlParam};
use sea_orm::DatabaseConnection;

use super::executor::SqliteExecutor;

/// SQLite executor plus SeaORM connection for the same database.
///
/// Implements [`DatabaseExecutor`] by delegating to the inner executor.
/// Repositories can downcast via `as_any()` and use [`sea_conn`](Self::sea_conn) for ORM operations.
pub struct SqliteBackend {
    executor: SqliteExecutor,
    sea_conn: DatabaseConnection,
}

impl SqliteBackend {
    /// Creates a backend with the given executor and SeaORM connection.
    #[must_use]
    pub fn new(executor: SqliteExecutor, sea_conn: DatabaseConnection) -> Self {
        Self { executor, sea_conn }
    }

    /// Returns the SeaORM connection for the same database.
    #[must_use]
    pub fn sea_conn(&self) -> &DatabaseConnection {
        &self.sea_conn
    }
}

#[async_trait]
impl DatabaseExecutor for SqliteBackend {
    async fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<()> {
        self.executor.execute(sql, params).await
    }

    async fn query_one(
        &self,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<Option<Arc<dyn mcb_domain::ports::SqlRow>>> {
        self.executor.query_one(sql, params).await
    }

    async fn query_all(
        &self,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<Vec<Arc<dyn mcb_domain::ports::SqlRow>>> {
        self.executor.query_all(sql, params).await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
