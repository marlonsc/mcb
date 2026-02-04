//! Handler for the `memory_get_quality_gates` MCP tool

use crate::args::MemoryGetQualityGatesArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{
    MemoryFilter, ObservationType, QualityGateResult, QualityGateStatus,
};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `memory_get_quality_gates` tool.
pub struct GetQualityGatesHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct QualityGateItem {
    observation_id: String,
    created_at: i64,
    gate_name: String,
    status: String,
    message: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
    execution_id: Option<String>,
    session_id: Option<String>,
    repo_id: Option<String>,
    branch: Option<String>,
    commit: Option<String>,
}

#[derive(Serialize)]
struct QualityGateResponse {
    count: usize,
    quality_gates: Vec<QualityGateItem>,
}

impl GetQualityGatesHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryGetQualityGatesArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let status_filter = args
            .status
            .as_ref()
            .map(|value| value.parse::<QualityGateStatus>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let time_range = match (args.start_time, args.end_time) {
            (Some(start), Some(end)) => Some((start, end)),
            (Some(start), None) => Some((start, i64::MAX)),
            (None, Some(end)) => Some((i64::MIN, end)),
            (None, None) => None,
        };

        let filter = MemoryFilter {
            id: None,
            tags: None,
            observation_type: Some(ObservationType::QualityGate),
            session_id: args.session_id.clone(),
            repo_id: args.repo_id.clone(),
            time_range,
            branch: args.branch.clone(),
            commit: args.commit.clone(),
        };

        let query = build_quality_gate_query(&args.gate_name);
        let fetch_limit = args.limit.saturating_mul(5);

        match self
            .memory_service
            .search_memories(&query, Some(filter), fetch_limit)
            .await
        {
            Ok(results) => {
                let mut items: Vec<QualityGateItem> = results
                    .into_iter()
                    .filter_map(|result| {
                        let obs = result.observation;
                        let quality_gate = obs.metadata.quality_gate?;
                        if !quality_gate_matches(
                            &quality_gate,
                            &args.gate_name,
                            &status_filter,
                            &args.execution_id,
                        ) {
                            return None;
                        }

                        Some(QualityGateItem {
                            observation_id: obs.id,
                            created_at: obs.created_at,
                            gate_name: quality_gate.gate_name,
                            status: quality_gate.status.as_str().to_string(),
                            message: quality_gate.message,
                            timestamp: quality_gate.timestamp,
                            execution_id: quality_gate.execution_id,
                            session_id: obs.metadata.session_id,
                            repo_id: obs.metadata.repo_id,
                            branch: obs.metadata.branch,
                            commit: obs.metadata.commit,
                        })
                    })
                    .collect();

                items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                items.truncate(args.limit);

                let response = QualityGateResponse {
                    count: items.len(),
                    quality_gates: items,
                };
                ResponseFormatter::json_success(&response)
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get quality gates: {e}"
            ))])),
        }
    }
}

fn build_quality_gate_query(gate_name: &Option<String>) -> String {
    match gate_name {
        Some(name) => format!("quality gate {name}"),
        None => "quality gate".to_string(),
    }
}

fn quality_gate_matches(
    quality_gate: &QualityGateResult,
    gate_name: &Option<String>,
    status: &Option<QualityGateStatus>,
    execution_id: &Option<String>,
) -> bool {
    if let Some(name) = gate_name
        && &quality_gate.gate_name != name
    {
        return false;
    }

    if let Some(status_filter) = status
        && &quality_gate.status != status_filter
    {
        return false;
    }

    if let Some(execution_filter) = execution_id
        && quality_gate.execution_id.as_ref() != Some(execution_filter)
    {
        return false;
    }

    true
}
