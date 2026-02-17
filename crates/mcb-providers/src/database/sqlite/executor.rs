//! `SQLite` implementation of the database executor port.
//!
//! Uses the domain port [`DatabaseExecutor`] and [`SqlRow`]; repositories depend
//! on these traits and do not use sqlx directly.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{DatabaseExecutor, SqlParam, SqlRow};
use sqlx::Column;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

/// Row adapter that copies column values from a `SQLite` row so it can be returned
/// as `Arc<dyn SqlRow>` without holding a reference to the connection.
#[derive(Debug)]
struct SqliteMappedRow {
    strings: HashMap<String, Option<String>>,
    i64s: HashMap<String, Option<i64>>,
    f64s: HashMap<String, Option<f64>>,
}

impl SqliteMappedRow {
    fn from_sqlite_row(row: &SqliteRow) -> Result<Self> {
        let mut strings = HashMap::new();
        let mut i64s = HashMap::new();
        let mut f64s = HashMap::new();
        for (i, col) in row.columns().iter().enumerate() {
            let name = col.name().to_owned();
            if let Ok(v) = row.try_get::<String, _>(i) {
                strings.insert(name.clone(), Some(v));
            } else if let Ok(v) = row.try_get::<i64, _>(i) {
                i64s.insert(name.clone(), Some(v));
            } else if let Ok(v) = row.try_get::<f64, _>(i) {
                f64s.insert(name.clone(), Some(v));
            } else if let Ok(opt) = row.try_get::<Option<String>, _>(i) {
                strings.insert(name.clone(), opt);
            } else if let Ok(opt) = row.try_get::<Option<i64>, _>(i) {
                i64s.insert(name.clone(), opt);
            } else if let Ok(opt) = row.try_get::<Option<f64>, _>(i) {
                f64s.insert(name.clone(), opt);
            } else {
                strings.insert(name, None);
            }
        }
        Ok(Self {
            strings,
            i64s,
            f64s,
        })
    }
}

impl SqlRow for SqliteMappedRow {
    fn try_get_string(&self, name: &str) -> Result<Option<String>> {
        Ok(self
            .strings
            .get(name)
            .cloned()
            .flatten()
            .or_else(|| self.i64s.get(name).and_then(|n| n.map(|v| v.to_string()))))
    }

    fn try_get_i64(&self, name: &str) -> Result<Option<i64>> {
        Ok(self.i64s.get(name).copied().flatten().or_else(|| {
            self.strings
                .get(name)
                .and_then(|s| s.as_ref().and_then(|s| s.parse().ok()))
        }))
    }

    fn try_get_f64(&self, name: &str) -> Result<Option<f64>> {
        Ok(self.f64s.get(name).copied().flatten().or_else(|| {
            self.strings
                .get(name)
                .and_then(|s| s.as_ref().and_then(|s| s.parse().ok()))
        }))
    }
}

/// `SQLite` implementation of the database executor port.
pub struct SqliteExecutor {
    pool: sqlx::SqlitePool,
}

impl SqliteExecutor {
    /// Create an executor that uses the given pool.
    #[must_use]
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    async fn execute_impl(&self, sql: &str, params: &[SqlParam]) -> Result<()> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = match p {
                SqlParam::String(s) => q.bind(s.as_str()),
                SqlParam::I64(n) => q.bind(*n),
                SqlParam::Bool(b) => q.bind(*b),
                SqlParam::Null => q.bind(Option::<String>::None),
            };
        }
        q.execute(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source(format!("SQL execute failed: {sql}"), e))?;
        Ok(())
    }

    async fn query_one_impl(
        &self,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<Option<Arc<dyn SqlRow>>> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = match p {
                SqlParam::String(s) => q.bind(s.as_str()),
                SqlParam::I64(n) => q.bind(*n),
                SqlParam::Bool(b) => q.bind(*b),
                SqlParam::Null => q.bind(Option::<String>::None),
            };
        }
        let row = q
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source(format!("SQL query_one failed: {sql}"), e))?;
        match row {
            Some(r) => {
                let map_row = SqliteMappedRow::from_sqlite_row(&r)
                    .map_err(|e| Error::memory_with_source("Failed to map row", e))?;
                Ok(Some(Arc::new(map_row) as Arc<dyn SqlRow>))
            }
            None => Ok(None),
        }
    }

    async fn query_all_impl(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<Arc<dyn SqlRow>>> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = match p {
                SqlParam::String(s) => q.bind(s.as_str()),
                SqlParam::I64(n) => q.bind(*n),
                SqlParam::Bool(b) => q.bind(*b),
                SqlParam::Null => q.bind(Option::<String>::None),
            };
        }
        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source(format!("SQL query_all failed: {sql}"), e))?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let map_row = SqliteMappedRow::from_sqlite_row(&r)
                .map_err(|e| Error::memory_with_source("Failed to map row", e))?;
            out.push(Arc::new(map_row) as Arc<dyn SqlRow>);
        }
        Ok(out)
    }
}

#[async_trait]
impl DatabaseExecutor for SqliteExecutor {
    async fn execute(&self, sql: &str, params: &[SqlParam]) -> Result<()> {
        self.execute_impl(sql, params).await
    }

    async fn query_one(&self, sql: &str, params: &[SqlParam]) -> Result<Option<Arc<dyn SqlRow>>> {
        self.query_one_impl(sql, params).await
    }

    async fn query_all(&self, sql: &str, params: &[SqlParam]) -> Result<Vec<Arc<dyn SqlRow>>> {
        self.query_all_impl(sql, params).await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SqliteExecutor {
    /// Get reference to inner pool
    #[must_use]
    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}
