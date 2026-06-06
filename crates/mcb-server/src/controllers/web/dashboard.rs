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

    let idx_run = count_active_indexing(&state);
    let val_run = count_active_validation(&state);

    let emb_ok = state.embedding_provider.health_check().await.is_ok();
    let vec_ok = state.vector_store.health_check().await.is_ok();
    let health = if emb_ok && vec_ok {
        "healthy"
    } else {
        "degraded"
    };

    let tool_rows =
        render_label_count_rows(tool_calls.iter().take(8).map(|t| (&t.tool_name, t.count)));
    let obs_rows = render_label_count_rows(daily.iter().map(|d| (&d.day, d.count)));

    let body = render_dashboard_body(DashboardView {
        sessions,
        agents,
        health,
        idx_run,
        val_run,
        tool_rows: &tool_rows,
        obs_rows: &obs_rows,
    });
    html_page!("Dashboard", body)
}

/// View-model carrying the computed values rendered by [`render_dashboard_body`].
#[derive(Clone, Copy)]
struct DashboardView<'a> {
    sessions: i64,
    agents: i64,
    health: &'a str,
    idx_run: usize,
    val_run: usize,
    tool_rows: &'a str,
    obs_rows: &'a str,
}

/// Assemble the dashboard HTML body from precomputed metrics and table rows.
fn render_dashboard_body(view: DashboardView<'_>) -> String {
    let DashboardView {
        sessions,
        agents,
        health,
        idx_run,
        val_run,
        tool_rows,
        obs_rows,
    } = view;
    format!(
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
    )
}

/// Count indexing operations currently starting or in progress.
fn count_active_indexing(state: &McbState) -> usize {
    state
        .indexing_ops
        .get_operations()
        .values()
        .filter(|o| {
            matches!(
                o.status,
                IndexingOperationStatus::Starting | IndexingOperationStatus::InProgress
            )
        })
        .count()
}

/// Count validation operations currently queued or in progress.
fn count_active_validation(state: &McbState) -> usize {
    state
        .validation_ops
        .get_operations()
        .values()
        .filter(|o| {
            matches!(
                o.status,
                ValidationStatus::Queued | ValidationStatus::InProgress
            )
        })
        .count()
}

/// Render `(label, count)` pairs as escaped two-column table rows.
fn render_label_count_rows<'a>(rows: impl Iterator<Item = (&'a String, i64)>) -> String {
    rows.map(|(label, count)| {
        format!(
            "<tr><td>{}</td><td class=\"num\">{}</td></tr>",
            html_escape(label),
            count
        )
    })
    .collect::<Vec<_>>()
    .join("\n")
}
