use std::sync::Arc;

use mcb_domain::entities::project::ProjectDependency;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use tracing::debug;

use crate::database::sqlite::row_convert;

pub(crate) async fn add_dependency(
    executor: &Arc<dyn DatabaseExecutor>,
    dep: &ProjectDependency,
) -> Result<()> {
    let params = [
        SqlParam::String(dep.id.clone()),
        SqlParam::String(dep.from_issue_id.clone()),
        SqlParam::String(dep.to_issue_id.clone()),
        SqlParam::String(dep.dependency_type.as_str().to_string()),
        SqlParam::I64(dep.created_at),
    ];

    executor
        .execute(
            r"
            INSERT INTO project_dependencies (
                id, from_issue_id, to_issue_id, dependency_type, created_at
            ) VALUES (?, ?, ?, ?, ?)
            ",
            &params,
        )
        .await?;

    debug!("Added dependency: {}", dep.id);
    Ok(())
}

pub(crate) async fn remove_dependency(
    executor: &Arc<dyn DatabaseExecutor>,
    id: &str,
) -> Result<()> {
    executor
        .execute(
            "DELETE FROM project_dependencies WHERE id = ?",
            &[SqlParam::String(id.to_string())],
        )
        .await?;

    debug!("Removed dependency: {}", id);
    Ok(())
}

pub(crate) async fn list_dependencies(
    executor: &Arc<dyn DatabaseExecutor>,
    project_id: &str,
) -> Result<Vec<ProjectDependency>> {
    let rows = executor
        .query_all(
            r"
            SELECT pd.* FROM project_dependencies pd
            JOIN project_issues pi ON pd.from_issue_id = pi.id
            WHERE pi.project_id = ?
            ORDER BY pd.created_at DESC
            ",
            &[SqlParam::String(project_id.to_string())],
        )
        .await?;

    let mut deps = Vec::with_capacity(rows.len());
    for row in rows {
        deps.push(
            row_convert::row_to_project_dependency(row.as_ref())
                .map_err(|e| Error::memory_with_source("decode project dependency row", e))?,
        );
    }
    Ok(deps)
}
