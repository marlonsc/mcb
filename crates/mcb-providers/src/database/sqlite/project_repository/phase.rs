use std::sync::Arc;

use mcb_domain::entities::project::ProjectPhase;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use tracing::debug;

use crate::database::sqlite::row_convert;

pub(crate) async fn create_phase(
    executor: &Arc<dyn DatabaseExecutor>,
    phase: &ProjectPhase,
) -> Result<()> {
    let params = [
        SqlParam::String(phase.id.clone()),
        SqlParam::String(phase.project_id.clone()),
        SqlParam::String(phase.name.clone()),
        SqlParam::String(phase.description.clone()),
        SqlParam::I64(phase.sequence as i64),
        SqlParam::String(phase.status.as_str().to_string()),
        phase.started_at.map_or(SqlParam::Null, SqlParam::I64),
        phase.completed_at.map_or(SqlParam::Null, SqlParam::I64),
        SqlParam::I64(phase.created_at),
        SqlParam::I64(phase.updated_at),
    ];

    executor
        .execute(
            r"
            INSERT INTO project_phases (
                id, project_id, name, description, sequence, status,
                started_at, completed_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ",
            &params,
        )
        .await?;

    debug!("Created phase: {}", phase.id);
    Ok(())
}

pub(crate) async fn get_phase(
    executor: &Arc<dyn DatabaseExecutor>,
    id: &str,
) -> Result<Option<ProjectPhase>> {
    let row = executor
        .query_one(
            "SELECT * FROM project_phases WHERE id = ?",
            &[SqlParam::String(id.to_string())],
        )
        .await?;

    match row {
        Some(r) => Ok(Some(
            row_convert::row_to_project_phase(r.as_ref())
                .map_err(|e| Error::memory_with_source("decode project phase row", e))?,
        )),
        None => Ok(None),
    }
}

pub(crate) async fn update_phase(
    executor: &Arc<dyn DatabaseExecutor>,
    phase: &ProjectPhase,
) -> Result<()> {
    let params = [
        SqlParam::String(phase.name.clone()),
        SqlParam::String(phase.description.clone()),
        SqlParam::I64(phase.sequence as i64),
        SqlParam::String(phase.status.as_str().to_string()),
        phase.started_at.map_or(SqlParam::Null, SqlParam::I64),
        phase.completed_at.map_or(SqlParam::Null, SqlParam::I64),
        SqlParam::I64(phase.updated_at),
        SqlParam::String(phase.id.clone()),
    ];

    executor
        .execute(
            r"
            UPDATE project_phases
            SET name = ?, description = ?, sequence = ?, status = ?,
                started_at = ?, completed_at = ?, updated_at = ?
            WHERE id = ?
            ",
            &params,
        )
        .await?;

    debug!("Updated phase: {}", phase.id);
    Ok(())
}

pub(crate) async fn list_phases(
    executor: &Arc<dyn DatabaseExecutor>,
    project_id: &str,
) -> Result<Vec<ProjectPhase>> {
    super::helpers::query_and_convert_all(
        executor,
        "SELECT * FROM project_phases WHERE project_id = ? ORDER BY sequence ASC",
        &[SqlParam::String(project_id.to_string())],
        row_convert::row_to_project_phase,
    )
    .await
}
