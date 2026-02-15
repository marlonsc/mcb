#![allow(dead_code)]
use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};

/// Helper to query a single row and convert it to an entity.
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
#[allow(dead_code)]
pub async fn insert(
    executor: &Arc<dyn DatabaseExecutor>,
    sql: &str,
    params: &[SqlParam],
) -> Result<i64> {
    executor.execute(sql, params).await?;
    // SQLite-specific: get last inserted rowid
    // We alias it to 'id' so we can look it up by name as required by SqlRow trait.
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
#[allow(dead_code)]
pub async fn update(
    executor: &Arc<dyn DatabaseExecutor>,
    sql: &str,
    params: &[SqlParam],
) -> Result<()> {
    executor.execute(sql, params).await
}

/// Helper to execute a DELETE statement.
#[allow(dead_code)]
pub async fn delete(
    executor: &Arc<dyn DatabaseExecutor>,
    sql: &str,
    params: &[SqlParam],
) -> Result<()> {
    executor.execute(sql, params).await
}

/// Helper to execute an UPSERT (INSERT OR REPLACE) statement.
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
