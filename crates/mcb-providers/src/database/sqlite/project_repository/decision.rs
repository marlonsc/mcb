use crate::database::sqlite::row_convert;
use mcb_domain::entities::project::ProjectDecision;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use std::sync::Arc;
use tracing::debug;

pub(crate) async fn create_decision(
    executor: &Arc<dyn DatabaseExecutor>,
    decision: &ProjectDecision,
) -> Result<()> {
    let params = [
        SqlParam::String(decision.id.clone()),
        SqlParam::String(decision.project_id.clone()),
        decision
            .issue_id
            .as_ref()
            .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        SqlParam::String(decision.title.clone()),
        SqlParam::String(decision.context.clone()),
        SqlParam::String(decision.decision.clone()),
        SqlParam::String(decision.consequences.clone()),
        SqlParam::I64(decision.created_at),
    ];

    executor
        .execute(
            r"
            INSERT INTO project_decisions (
                id, project_id, issue_id, title, context, decision, consequences, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ",
            &params,
        )
        .await?;

    debug!("Created decision: {}", decision.id);
    Ok(())
}

pub(crate) async fn get_decision(
    executor: &Arc<dyn DatabaseExecutor>,
    id: &str,
) -> Result<Option<ProjectDecision>> {
    let row = executor
        .query_one(
            "SELECT * FROM project_decisions WHERE id = ?",
            &[SqlParam::String(id.to_string())],
        )
        .await?;

    match row {
        Some(r) => Ok(Some(
            row_convert::row_to_project_decision(r.as_ref())
                .map_err(|e| Error::memory_with_source("decode project decision row", e))?,
        )),
        None => Ok(None),
    }
}

pub(crate) async fn list_decisions(
    executor: &Arc<dyn DatabaseExecutor>,
    project_id: &str,
) -> Result<Vec<ProjectDecision>> {
    let rows = executor
        .query_all(
            "SELECT * FROM project_decisions WHERE project_id = ? ORDER BY created_at DESC",
            &[SqlParam::String(project_id.to_string())],
        )
        .await?;

    let mut decisions = Vec::with_capacity(rows.len());
    for row in rows {
        decisions.push(
            row_convert::row_to_project_decision(row.as_ref())
                .map_err(|e| Error::memory_with_source("decode project decision row", e))?,
        );
    }
    Ok(decisions)
}
