use crate::{controllers::admin_config::load_admin_config, state::McbState};
use axum::extract::Extension;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

/// JSON body for dashboard query requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardBody {
    /// Graph identifier (e.g. `observations_by_month`, `sessions_by_day`).
    pub graph: String,
    /// Optional row limit.
    pub limit: Option<usize>,
}

/// Key-value pair for dashboard series data.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Datum {
    /// Label (e.g. month, day, tool name).
    pub key: String,
    /// Count or value.
    pub val: i64,
}

fn datum(key: String, val: i64) -> Datum {
    Datum { key, val }
}

/// Returns admin config as JSON (sea-orm-pro).
///
/// # Errors
///
/// Fails when config cannot be loaded or serialized.
pub async fn config(Extension(_state): Extension<McbState>) -> Result<Response> {
    format::json(load_admin_config()?)
}

/// Returns dashboard series data for the requested graph.
///
/// # Errors
///
/// Fails when the dashboard port fails or graph is unknown.
pub async fn dashboard(
    Extension(state): Extension<McbState>,
    Json(body): Json<DashboardBody>,
) -> Result<Response> {
    let limit = body.limit.unwrap_or(30);
    let data = match body.graph.as_str() {
        "observations_by_month" => state
            .dashboard
            .get_observations_by_month(limit)
            .await
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?
            .into_iter()
            .map(|it| datum(it.month, it.count))
            .collect::<Vec<_>>(),
        "sessions_by_day" | "observations_by_day" => state
            .dashboard
            .get_observations_by_day(limit)
            .await
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?
            .into_iter()
            .map(|it| datum(it.day, it.count))
            .collect::<Vec<_>>(),
        "tool_calls_by_tool" => state
            .dashboard
            .get_tool_call_counts()
            .await
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?
            .into_iter()
            .map(|it| datum(it.tool_name, it.count))
            .collect::<Vec<_>>(),
        "agent_session_stats" => {
            let stats = state
                .dashboard
                .get_agent_session_stats()
                .await
                .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
            vec![
                datum("total_sessions".to_owned(), stats.total_sessions),
                datum("total_agents".to_owned(), stats.total_agents),
            ]
        }
        _ => not_found()?,
    };
    format::json(data)
}

/// Registers admin routes under `/admin`.
#[must_use]
pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin")
        .add("/config", get(config))
        .add("/dashboard", post(dashboard))
}
