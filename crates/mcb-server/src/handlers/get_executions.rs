//! Handler for the `memory_get_executions` MCP tool

use crate::args::MemoryGetExecutionsArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{
    ExecutionMetadata, ExecutionType, MemoryFilter, ObservationType,
};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `memory_get_executions` tool.
pub struct GetExecutionsHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct ExecutionItem {
    observation_id: String,
    created_at: i64,
    command: String,
    execution_type: String,
    success: bool,
    exit_code: Option<i32>,
    duration_ms: Option<i64>,
    coverage: Option<f32>,
    files_affected: Vec<String>,
    output_summary: Option<String>,
    warnings_count: Option<i32>,
    errors_count: Option<i32>,
    session_id: Option<String>,
    repo_id: Option<String>,
    branch: Option<String>,
    commit: Option<String>,
}

#[derive(Serialize)]
struct ExecutionResponse {
    count: usize,
    executions: Vec<ExecutionItem>,
}

impl GetExecutionsHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryGetExecutionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let execution_type_filter = args
            .execution_type
            .as_ref()
            .map(|value| value.parse::<ExecutionType>())
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
            observation_type: Some(ObservationType::Execution),
            session_id: args.session_id.clone(),
            repo_id: args.repo_id.clone(),
            time_range,
            branch: args.branch.clone(),
            commit: args.commit.clone(),
        };

        let query = build_execution_query(&execution_type_filter);
        let fetch_limit = args.limit.saturating_mul(5);

        match self
            .memory_service
            .search_memories(&query, Some(filter), fetch_limit)
            .await
        {
            Ok(results) => {
                let mut items: Vec<ExecutionItem> = results
                    .into_iter()
                    .filter_map(|result| {
                        let obs = result.observation;
                        let execution = obs.metadata.execution?;
                        if !execution_matches(&execution, &execution_type_filter, args.success) {
                            return None;
                        }

                        Some(ExecutionItem {
                            observation_id: obs.id,
                            created_at: obs.created_at,
                            command: execution.command,
                            execution_type: execution.execution_type.as_str().to_string(),
                            success: execution.success,
                            exit_code: execution.exit_code,
                            duration_ms: execution.duration_ms,
                            coverage: execution.coverage,
                            files_affected: execution.files_affected,
                            output_summary: execution.output_summary,
                            warnings_count: execution.warnings_count,
                            errors_count: execution.errors_count,
                            session_id: obs.metadata.session_id,
                            repo_id: obs.metadata.repo_id,
                            branch: obs.metadata.branch,
                            commit: obs.metadata.commit,
                        })
                    })
                    .collect();

                items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                items.truncate(args.limit);

                let response = ExecutionResponse {
                    count: items.len(),
                    executions: items,
                };

                let json = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| String::from("Failed to serialize results"));

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get executions: {e}"
            ))])),
        }
    }
}

fn build_execution_query(execution_type: &Option<ExecutionType>) -> String {
    match execution_type {
        Some(kind) => format!("execution {}", kind.as_str()),
        None => "execution".to_string(),
    }
}

fn execution_matches(
    execution: &ExecutionMetadata,
    execution_type: &Option<ExecutionType>,
    success: Option<bool>,
) -> bool {
    if let Some(kind) = execution_type
        && &execution.execution_type != kind
    {
        return false;
    }
    if let Some(success_filter) = success
        && execution.success != success_filter
    {
        return false;
    }

    true
}
