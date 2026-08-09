//! Jobs page — shows indexing and validation operations.

use std::collections::HashMap;

use super::{data_table, html_escape, html_page, metric_card, status_badge};
use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;
use mcb_domain::ports::{
    IndexingOperation, IndexingOperationStatus, ValidationOperation, ValidationStatus,
};
use mcb_domain::value_objects::ids::OperationId;

/// Jobs page handler.
///
/// # Errors
///
/// Fails when job data cannot be loaded.
pub async fn jobs_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let idx_ops = state.indexing_ops.get_operations();
    let val_ops = state.validation_ops.get_operations();

    let running = idx_ops
        .values()
        .filter(|o| {
            matches!(
                o.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count()
        + val_ops
            .values()
            .filter(|o| {
                matches!(
                    o.status,
                    ValidationStatus::Queued | ValidationStatus::InProgress
                )
            })
            .count();
    let total = idx_ops.len() + val_ops.len();
    let status_str = if running > 0 { "running" } else { "idle" };

    let idx_rows = render_indexing_rows(&idx_ops);
    let val_rows = render_validation_rows(&val_ops);
    let all_rows = format!("{idx_rows}{val_rows}");

    let body = format!(
        r#"<h1>Jobs</h1>
<div class="dashboard-grid">{}{}{}{}</div>
<div class="card"><h3>Operations</h3>{}</div>"#,
        metric_card!("Status", status_badge!(status_str, status_str), ""),
        metric_card!("Total", total, ""),
        metric_card!("Running", running, ""),
        metric_card!("Complete", total - running, ""),
        data_table!(
            "<th>ID</th><th>Type</th><th>Status</th><th>Files</th>",
            all_rows,
            "No operations recorded."
        ),
    );
    html_page!("Jobs", body)
}

/// Render indexing operations as `Operations` table rows.
fn render_indexing_rows(ops: &HashMap<OperationId, IndexingOperation>) -> String {
    ops.iter()
        .map(|(id, op)| {
            let cls = match op.status {
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress => {
                    "running"
                }
                IndexingOperationStatus::Completed => "completed",
                _ => "error",
            };
            format!(
                "<tr><td>{}</td><td>indexing</td><td>{}</td><td class=\"num\">{}</td></tr>",
                html_escape(&id.to_string()),
                status_badge!(cls, format!("{:?}", op.status)),
                op.total_files,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render validation operations as `Operations` table rows.
fn render_validation_rows(ops: &HashMap<OperationId, ValidationOperation>) -> String {
    ops.iter()
        .map(|(id, op)| {
            let cls = match op.status {
                ValidationStatus::Queued | ValidationStatus::InProgress => "running",
                ValidationStatus::Completed => "completed",
                ValidationStatus::Failed(_) | ValidationStatus::Canceled => "error",
            };
            format!(
                "<tr><td>{}</td><td>validation</td><td>{}</td><td class=\"num\">—</td></tr>",
                html_escape(&id.to_string()),
                status_badge!(cls, format!("{:?}", op.status)),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
