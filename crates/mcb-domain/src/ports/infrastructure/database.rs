//! Database executor port (infrastructure).
//!
//! Abstraction for SQL execution so repositories and application code do not
//! depend on a concrete driver (e.g. SQLite/sqlx). Implementations live in
//! infrastructure and are injected via DI.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;

/// Parameter for prepared statement binding (driver-agnostic).
#[derive(Debug, Clone)]
pub enum SqlParam {
    /// String value
    String(String),
    /// 64-bit integer
    I64(i64),
    /// Boolean value
    Bool(bool),
    /// Null
    Null,
}

/// Abstraction for a single query result row.
///
/// Implementations wrap driver-specific rows (e.g. `sqlx::SqliteRow`) and expose
/// values by column name so repository code can map to domain entities without
/// depending on the driver.
pub trait SqlRow: Send + Sync {
    /// Try to get a string by column name.
    ///
    /// # Errors
    ///
    /// Returns an error if the column does not exist or the value cannot be
    /// decoded as a string.
    fn try_get_string(&self, name: &str) -> Result<Option<String>>;

    /// Try to get an i64 by column name.
    ///
    /// # Errors
    ///
    /// Returns an error if the column does not exist or the value cannot be
    /// decoded as an i64.
    fn try_get_i64(&self, name: &str) -> Result<Option<i64>>;

    /// Try to get an f64 by column name (e.g. FTS rank).
    ///
    /// # Errors
    ///
    /// Returns an error if the column does not exist or the value cannot be
    /// decoded as an f64.
    fn try_get_f64(&self, name: &str) -> Result<Option<f64>>;
}

/// Port for executing SQL (infrastructure capability).
#[async_trait]
pub trait DatabaseExecutor: Send + Sync {
    /// Performs the execute operation.
    async fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<()>;

    /// Performs the query one operation.
    async fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<Arc<dyn SqlRow>>>;

    /// Performs the query all operation.
    async fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<Arc<dyn SqlRow>>>;

    /// Cast to Any to allow downcasting to concrete type (e.g. `SqlitePool`) for internal use
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Provider factory for database connections with schema initialization.
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// Performs the connect operation.
    async fn connect(&self, path: &std::path::Path) -> Result<Arc<dyn DatabaseExecutor>>;
}
