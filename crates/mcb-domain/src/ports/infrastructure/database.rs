//! Database executor port (infrastructure).
//!
//! Abstraction for SQL execution so repositories and application code do not
//! depend on a concrete driver (e.g. SQLite/sqlx). Implementations live in
//! infrastructure and are injected via DI.

use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Parameter for prepared statement binding (driver-agnostic).
#[derive(Debug, Clone)]
pub enum SqlParam {
    /// String value
    String(String),
    /// 64-bit integer
    I64(i64),
    /// Null
    Null,
}

/// Abstraction for a single query result row.
///
/// Implementations wrap driver-specific rows (e.g. sqlx::SqliteRow) and expose
/// values by column name so repository code can map to domain entities without
/// depending on the driver.
pub trait SqlRow: Send + Sync {
    /// Try to get a string by column name.
    fn try_get_string(&self, name: &str) -> Result<Option<String>>;

    /// Try to get an i64 by column name.
    fn try_get_i64(&self, name: &str) -> Result<Option<i64>>;

    /// Try to get an f64 by column name (e.g. FTS rank).
    fn try_get_f64(&self, name: &str) -> Result<Option<f64>>;
}

/// Port for executing SQL (infrastructure capability).
///
/// Repositories depend on this trait via DI; they do not hold pools or use
/// driver types directly. Implementations (e.g. SQLite in infrastructure) perform
/// the actual execution.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
/// use std::sync::Arc;
///
/// async fn run_query(exec: Arc<dyn DatabaseExecutor>) -> mcb_domain::Result<()> {
///     exec.execute(
///         "INSERT INTO t (id) VALUES (?)",
///         &[SqlParam::String("x".into())],
///     )
///     .await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait DatabaseExecutor: Send + Sync {
    async fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<()>;

    async fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<Arc<dyn SqlRow>>>;

    async fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<Arc<dyn SqlRow>>>;
}

/// Provider factory for database connections with schema initialization.
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    async fn connect(&self, path: &std::path::Path) -> Result<Arc<dyn DatabaseExecutor>>;

    async fn connect_in_memory(&self) -> Result<Arc<dyn DatabaseExecutor>>;
}
