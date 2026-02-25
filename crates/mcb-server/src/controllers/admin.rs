use crate::{constants::admin::CONFIG_ROOT, state::McbState};
use axum::extract::Extension;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardBody {
    pub graph: String,
    pub limit: Option<usize>,
}
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Datum {
    pub key: String,
    pub val: i64,
}

fn datum(key: String, val: i64) -> Datum {
    Datum { key, val }
}
pub async fn config(Extension(_state): Extension<McbState>) -> Result<Response> {
    let config = sea_orm_pro::ConfigParser::new()
        .load_config(CONFIG_ROOT)
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
    format::json(config)
}
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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin")
        .add("/config", get(config))
        .add("/dashboard", post(dashboard))
}
