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
