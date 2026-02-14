use std::sync::Arc;

use mcb_domain::entities::memory::{
    MemoryFilter, ObservationMetadata, ObservationType, QualityGateResult,
};
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::{compute_stable_id_hash, vcs_context::VcsContext};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;
use uuid::Uuid;

use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{OriginContextInput, resolve_origin_context};
use crate::utils::json;

/// Stores a quality gate result as a semantic observation.
#[tracing::instrument(skip_all)]
pub async fn store_quality_gate(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match json::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for quality gate store",
            )]));
        }
    };
    let gate_name = match data.get("gate_name").and_then(Value::as_str) {
        Some(value) => value.to_owned(),
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing required field: gate_name",
            )]));
        }
    };
    let status_str = match data.get("status").and_then(Value::as_str) {
        Some(value) => value.to_owned(),
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing required field: status",
            )]));
        }
    };
    let status: mcb_domain::entities::memory::QualityGateStatus = match status_str.parse() {
        Ok(v) => v,
        Err(error) => {
            return Ok(CallToolResult::error(vec![Content::text(
                error.to_string(),
            )]));
        }
    };
    let timestamp = data
        .get("timestamp")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    let quality_gate = QualityGateResult {
        id: Uuid::new_v4().to_string(),
        gate_name: gate_name.clone(),
        status,
        message: data
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_owned),
        timestamp,
        execution_id: data
            .get("execution_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
    };
    let vcs_context = VcsContext::capture();
    let content = format!(
        "Quality Gate: {} (status={})",
        gate_name,
        quality_gate.status.as_str()
    );
    let tags = vec![
        "quality_gate".to_string(),
        quality_gate.status.as_str().to_string(),
    ];
    let arg_session_id = args
        .session_id
        .clone()
        .map(|id| compute_stable_id_hash("session", id.as_str()));
    let canonical_session_id = arg_session_id.clone();
    let parent_session_hash = args
        .parent_session_id
        .clone()
        .map(|id| compute_stable_id_hash("parent_session", id.as_str()));
    let payload_repo_id = data
        .get("repo_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_project_id = data
        .get("project_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_branch = data
        .get("branch")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_commit = data
        .get("commit")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_repo_path = data
        .get("repo_path")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_worktree_id = data
        .get("worktree_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_operator_id = data
        .get("operator_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_machine_id = data
        .get("machine_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_agent_program = data
        .get("agent_program")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_model_id = data
        .get("model_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_delegated = data.get("delegated").and_then(Value::as_bool);

    let mut origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        session_from_args: None,
        session_from_data: None,
        parent_session_from_args: None,
        parent_session_from_data: None,
        execution_from_args: quality_gate.execution_id.as_deref(),
        execution_from_data: quality_gate.execution_id.as_deref(),
        tool_name_args: Some("memory"),
        tool_name_payload: None,
        repo_id_args: args.repo_id.as_deref(),
        repo_id_payload: payload_repo_id.as_deref(),
        repo_path_args: None,
        repo_path_payload: payload_repo_path.as_deref(),
        worktree_id_args: None,
        worktree_id_payload: payload_worktree_id.as_deref(),
        file_path_args: None,
        file_path_payload: None,
        branch_args: None,
        branch_payload: payload_branch.as_deref(),
        commit_args: None,
        commit_payload: payload_commit.as_deref(),
        operator_id_args: None,
        operator_id_payload: payload_operator_id.as_deref(),
        machine_id_args: None,
        machine_id_payload: payload_machine_id.as_deref(),
        agent_program_args: None,
        agent_program_payload: payload_agent_program.as_deref(),
        model_id_args: None,
        model_id_payload: payload_model_id.as_deref(),
        delegated_args: None,
        delegated_payload: payload_delegated,
        require_project_id: true,
        timestamp: Some(timestamp),
    })?;
    if origin_context.repo_id.is_none() {
        origin_context.repo_id = vcs_context.repo_id.clone();
    }
    if origin_context.branch.is_none() {
        origin_context.branch = vcs_context.branch.clone();
    }
    if origin_context.commit.is_none() {
        origin_context.commit = vcs_context.commit.clone();
    }
    origin_context.session_id = None;
    origin_context.session_id_hash = canonical_session_id.clone();
    origin_context.parent_session_id = None;
    origin_context.parent_session_id_hash = parent_session_hash;
    let project_id = origin_context.project_id.clone().ok_or_else(|| {
        McpError::invalid_params("project_id is required for quality gate store", None)
    })?;

    let obs_metadata = ObservationMetadata {
        id: Uuid::new_v4().to_string(),
        session_id: canonical_session_id,
        repo_id: origin_context.repo_id.clone(),
        file_path: None,
        branch: origin_context.branch.clone(),
        commit: origin_context.commit.clone(),
        execution: None,
        quality_gate: Some(quality_gate),
        origin_context: Some(origin_context),
    };

    match memory_service
        .store_observation(
            project_id,
            content,
            ObservationType::QualityGate,
            tags,
            obs_metadata,
        )
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}

/// Retrieves stored quality gate results based on filters.
#[tracing::instrument(skip_all)]
pub async fn get_quality_gates(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = MemoryFilter {
        id: None,
        project_id: args.project_id.clone(),
        tags: None,
        r#type: Some(ObservationType::QualityGate),
        session_id: args
            .session_id
            .clone()
            .map(|id| compute_stable_id_hash("session", id.as_str())),
        parent_session_id: args.parent_session_id.clone(),
        repo_id: args.repo_id.clone(),
        time_range: None,
        branch: None,
        commit: None,
    };
    let query = "quality gate";
    let limit = args.limit.unwrap_or(10) as usize;
    let fetch_limit = limit * 5;
    match memory_service
        .search_memories(query, Some(filter), fetch_limit)
        .await
    {
        Ok(results) => {
            let mut gates: Vec<_> = results
                .into_iter()
                .filter_map(|result| {
                    // Extract quality gate metadata from observation; skip if missing
                    // (None indicates observation is not a quality gate type)
                    if let Some(gate) = result.observation.metadata.quality_gate.as_ref() {
                        Some(serde_json::json!({
                            "observation_id": result.observation.id,
                            "gate_name": gate.gate_name,
                            "status": gate.status.as_str(),
                            "message": gate.message,
                            "timestamp": gate.timestamp,
                            "execution_id": gate.execution_id,
                            "created_at": result.observation.created_at,
                        }))
                    } else {
                        None
                    }
                })
                .collect();
            gates.sort_by(|a, b| {
                b.get("created_at")
                    .and_then(|v| v.as_i64())
                    .cmp(&a.get("created_at").and_then(|v| v.as_i64()))
            });
            gates.truncate(limit);
            ResponseFormatter::json_success(&serde_json::json!({
                "count": gates.len(),
                "quality_gates": gates,
            }))
        }
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
