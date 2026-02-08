use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};

/// Generic helper to convert query results into domain entities
pub(crate) async fn query_and_convert_all<T, F>(
    executor: &Arc<dyn DatabaseExecutor>,
    sql: &str,
    params: &[SqlParam],
    converter: F,
) -> Result<Vec<T>>
where
    F: Fn(&dyn SqlRow) -> Result<T>,
{
    let rows = executor.query_all(sql, params).await?;
    let mut entities = Vec::with_capacity(rows.len());
    for row in rows {
        entities.push(converter(row.as_ref())?);
    }
    Ok(entities)
}
