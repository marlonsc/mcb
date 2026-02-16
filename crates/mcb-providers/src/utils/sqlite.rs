//! SQLite Provider Utilities
//!
//! Shared row mapping, query helpers, and extraction functions
//! used across SQLite repository implementations.

/// Row extraction helpers for mapping `SqlRow` values to Rust types.
pub mod row {
    use mcb_domain::error::{Error, Result};
    use mcb_domain::ports::infrastructure::database::{SqlParam, SqlRow};

    /// Helper to get a required string field.
    ///
    /// # Errors
    ///
    /// Returns an error if the column is missing or cannot be read.
    pub fn req_str(row: &dyn SqlRow, col: &str) -> Result<String> {
        row.try_get_string(col)?
            .ok_or_else(|| Error::memory(format!("Missing {col}")))
    }

    /// Helper to get a required i64 field.
    ///
    /// # Errors
    ///
    /// Returns an error if the column is missing or cannot be read.
    pub fn req_i64(row: &dyn SqlRow, col: &str) -> Result<i64> {
        row.try_get_i64(col)?
            .ok_or_else(|| Error::memory(format!("Missing {col}")))
    }

    /// Helper to get an optional string field.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be read.
    pub fn opt_str(row: &dyn SqlRow, col: &str) -> Result<Option<String>> {
        row.try_get_string(col)
    }

    /// Helper to get an optional i64 field.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be read.
    pub fn opt_i64(row: &dyn SqlRow, col: &str) -> Result<Option<i64>> {
        row.try_get_i64(col)
    }

    /// Helper to convert Option<String> to `SqlParam`.
    #[must_use]
    pub fn opt_str_param(value: &Option<String>) -> SqlParam {
        match value {
            Some(v) => SqlParam::String(v.clone()),
            None => SqlParam::Null,
        }
    }

    /// Helper to convert Option<i64> to `SqlParam`.
    #[must_use]
    pub fn opt_i64_param(value: Option<i64>) -> SqlParam {
        match value {
            Some(v) => SqlParam::I64(v),
            None => SqlParam::Null,
        }
    }
}

/// Query helpers for common database operations.
pub mod query {
    #![allow(dead_code)]
    use std::sync::Arc;

    use mcb_domain::error::{Error, Result};
    use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};

    /// Helper to query a single row and convert it to an entity.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query or row conversion fails.
    pub async fn query_one<T, F>(
        executor: &Arc<dyn DatabaseExecutor>,
        sql: &str,
        params: &[SqlParam],
        convert: F,
    ) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

    /// Helper to query multiple rows and convert them to entities.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query or row conversion fails.
    pub async fn query_all<T, F>(
        executor: &Arc<dyn DatabaseExecutor>,
        sql: &str,
        params: &[SqlParam],
        convert: F,
        entity_name: &str,
    ) -> Result<Vec<T>>
    where
        F: Fn(&dyn SqlRow) -> Result<T>,
    {
        let rows = executor.query_all(sql, params).await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(
                convert(row.as_ref())
                    .map_err(|e| Error::memory_with_source(format!("decode {entity_name}"), e))?,
            );
        }
        Ok(result)
    }

    /// Helper to execute an INSERT statement and return the ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the SQL execution fails.
    #[allow(dead_code)]
    pub async fn insert(
        executor: &Arc<dyn DatabaseExecutor>,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<i64> {
        executor.execute(sql, params).await?;
        let row = executor
            .query_one("SELECT last_insert_rowid() as id", &[])
            .await?;
        if let Some(r) = row {
            Ok(r.try_get_i64("id")?.unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    /// Helper to execute an UPDATE statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the SQL execution fails.
    #[allow(dead_code)]
    pub async fn update(
        executor: &Arc<dyn DatabaseExecutor>,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<()> {
        executor.execute(sql, params).await
    }

    /// Helper to execute a DELETE statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the SQL execution fails.
    #[allow(dead_code)]
    pub async fn delete(
        executor: &Arc<dyn DatabaseExecutor>,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<()> {
        executor.execute(sql, params).await
    }

    /// Helper to execute an UPSERT (INSERT OR REPLACE) statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the SQL execution fails.
    #[allow(dead_code)]
    pub async fn upsert(
        executor: &Arc<dyn DatabaseExecutor>,
        sql: &str,
        params: &[SqlParam],
    ) -> Result<i64> {
        executor.execute(sql, params).await?;
        let row = executor
            .query_one("SELECT last_insert_rowid() as id", &[])
            .await?;
        if let Some(r) = row {
            Ok(r.try_get_i64("id")?.unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
