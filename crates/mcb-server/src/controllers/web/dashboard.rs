//! Dashboard page — landing page with real metrics.

use super::{data_table, html_escape, html_page, metric_card, status_badge};
use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;
use mcb_domain::ports::{IndexingOperationStatus, ValidationStatus};

/// Dashboard page handler.
///
/// # Errors
///
/// Fails when dashboard data cannot be loaded.
pub async fn dashboard(Extension(state): Extension<McbState>) -> Result<Response> {
    let stats = state.dashboard.get_agent_session_stats().await.ok();
    let tool_calls = state
        .dashboard
        .get_tool_call_counts()
        .await
        .unwrap_or_default();
    let daily = state
        .dashboard
        .get_observations_by_day(7)
        .await
        .unwrap_or_default();

    let sessions = stats.as_ref().map_or(0, |s| s.total_sessions);
    let agents = stats.as_ref().map_or(0, |s| s.total_agents);

    let idx_run = state
        .indexing_ops
        .get_operations()
        .values()
        .filter(|o| {
            matches!(
                o.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count();
    let val_run = state
        .validation_ops
        .get_operations()
        .values()
        .filter(|o| {
            matches!(
                o.status,
                ValidationStatus::Queued | ValidationStatus::InProgress
            )
        })
        .count();

    let emb_ok = state.embedding_provider.health_check().await.is_ok();
    let vec_ok = state.vector_store.health_check().await.is_ok();
    let health = if emb_ok && vec_ok {
        "healthy"
    } else {
        "degraded"
    };

    let tool_rows: String = tool_calls
        .iter()
        .take(8)
        .map(|t| {
            format!(
                "<tr><td>{}</td><td class=\"num\">{}</td></tr>",
                html_escape(&t.tool_name),
                t.count
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let obs_rows: String = daily
        .iter()
        .map(|d| {
            format!(
                "<tr><td>{}</td><td class=\"num\">{}</td></tr>",
                html_escape(&d.day),
                d.count
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        r#"<h1>Dashboard</h1>
<div class="dashboard-grid">{}{}{}{}</div>
<div class="dashboard-grid two-col">
  <div class="card"><h3>Tool Usage</h3>{}</div>
  <div class="card"><h3>Observations (Last 7 Days)</h3>{}</div>
</div>"#,
        metric_card!("Sessions", sessions, "total sessions"),
        metric_card!("Agents", agents, "unique agents"),
        metric_card!(
            "System Health",
            status_badge!(health, health),
            "embedding + vector store"
        ),
        metric_card!(
            "Active Jobs",
            idx_run + val_run,
            format!("{idx_run} indexing, {val_run} validation")
        ),
        data_table!(
            "<th>Tool</th><th>Calls</th>",
            tool_rows,
            "No tool calls recorded yet."
        ),
        data_table!(
            "<th>Day</th><th>Count</th>",
            obs_rows,
            "No observations recorded yet."
        ),
    );
    html_page!("Dashboard", body)
}
