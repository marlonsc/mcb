use mcb_domain::entities::memory::{ExecutionMetadata, ObservationMetadata, QualityGateResult};
use std::sync::Arc;

use mcb_domain::utils::{compute_stable_id_hash, vcs_context::VcsContext};
use mcb_domain::{
    entities::memory::{MemoryFilter, MemorySearchResult, ObservationType},
    ports::services::MemoryServiceInterface,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::args::MemoryArgs;
use crate::handlers::helpers::{OriginContextInput, resolve_origin_context};
pub(super) use crate::handlers::helpers::{
    opt_bool, opt_str, require_data_map, require_str, str_vec,
};

pub(super) fn require_i64(data: &Map<String, Value>, key: &str) -> Result<i64, CallToolResult> {
    data.get(key).and_then(Value::as_i64).ok_or_else(|| {
        CallToolResult::error(vec![Content::text(format!(
            "Missing required field: {key}"
        ))])
    })
}

pub(super) fn require_i32(data: &Map<String, Value>, key: &str) -> Result<i32, CallToolResult> {
    data.get(key)
        .and_then(Value::as_i64)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
}

pub(super) fn require_bool(data: &Map<String, Value>, key: &str) -> Result<bool, CallToolResult> {
    data.get(key).and_then(Value::as_bool).ok_or_else(|| {
        CallToolResult::error(vec![Content::text(format!(
            "Missing required field: {key}"
        ))])
    })
}

pub(super) struct MemoryOriginResolution {
    pub project_id: String,
    pub canonical_session_id: Option<String>,
    pub origin_context: mcb_domain::entities::memory::OriginContext,
}

pub(super) struct MemoryOriginOptions<'a> {
    pub execution_from_args: Option<&'a str>,
    pub execution_from_data: Option<&'a str>,
    pub file_path_payload: Option<&'a str>,
    pub timestamp: Option<i64>,
}

pub(super) fn resolve_memory_origin_context(
    args: &MemoryArgs,
    data: &Map<String, Value>,
    opts: MemoryOriginOptions<'_>,
) -> Result<MemoryOriginResolution, McpError> {
    let vcs_context = VcsContext::capture();
    let canonical_session_id = args
        .session_id
        .clone()
        .map(|id| compute_stable_id_hash("session", id.as_str()));
    let parent_session_hash = args
        .parent_session_id
        .clone()
        .map(|id| compute_stable_id_hash("parent_session", id.as_str()));

    let payload_repo_id = opt_str(data, "repo_id");
    let payload_project_id = opt_str(data, "project_id");
    let payload_branch = opt_str(data, "branch");
    let payload_commit = opt_str(data, "commit");
    let payload_repo_path = opt_str(data, "repo_path");
    let payload_worktree_id = opt_str(data, "worktree_id");
    let payload_operator_id = opt_str(data, "operator_id");
    let payload_machine_id = opt_str(data, "machine_id");
    let payload_agent_program = opt_str(data, "agent_program");
    let payload_model_id = opt_str(data, "model_id");
    let payload_delegated = opt_bool(data, "delegated");

    let mut origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        execution_from_args: opts.execution_from_args,
        execution_from_data: opts.execution_from_data,
        tool_name_args: Some("memory"),
        repo_id_args: args.repo_id.as_deref(),
        repo_id_payload: payload_repo_id.as_deref(),
        repo_path_payload: payload_repo_path.as_deref(),
        worktree_id_payload: payload_worktree_id.as_deref(),
        file_path_payload: opts.file_path_payload,
        branch_payload: payload_branch.as_deref(),
        commit_payload: payload_commit.as_deref(),
        operator_id_payload: payload_operator_id.as_deref(),
        machine_id_payload: payload_machine_id.as_deref(),
        agent_program_payload: payload_agent_program.as_deref(),
        model_id_payload: payload_model_id.as_deref(),
        delegated_payload: payload_delegated,
        require_project_id: true,
        timestamp: opts.timestamp,
        ..Default::default()
    })?;

    if origin_context.repo_id.is_none() {
        origin_context.repo_id = vcs_context.repo_id;
    }
    if origin_context.branch.is_none() {
        origin_context.branch = vcs_context.branch;
    }
    if origin_context.commit.is_none() {
        origin_context.commit = vcs_context.commit;
    }

    origin_context.session_id = None;
    origin_context.session_id_hash = canonical_session_id.clone();
    origin_context.parent_session_id = None;
    origin_context.parent_session_id_hash = parent_session_hash;

    let project_id = origin_context
        .project_id
        .clone()
        .ok_or_else(|| McpError::invalid_params("project_id is required for memory store", None))?;

    Ok(MemoryOriginResolution {
        project_id,
        canonical_session_id,
        origin_context,
    })
}

pub(super) fn build_observation_metadata(
    canonical_session_id: Option<String>,
    origin_context: mcb_domain::entities::memory::OriginContext,
    execution: Option<ExecutionMetadata>,
    quality_gate: Option<QualityGateResult>,
) -> ObservationMetadata {
    ObservationMetadata {
        id: Uuid::new_v4().to_string(),
        session_id: canonical_session_id,
        repo_id: origin_context.repo_id.clone(),
        file_path: origin_context.file_path.clone(),
        branch: origin_context.branch.clone(),
        commit: origin_context.commit.clone(),
        execution,
        quality_gate,
        origin_context: Some(origin_context),
    }
}

pub(super) async fn search_memories_as_json(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
    query: &'static str,
    obs_type: ObservationType,
    result_key: &'static str,
    mapper: impl Fn(&MemorySearchResult) -> Option<Value>,
) -> Result<CallToolResult, McpError> {
    let filter = build_memory_filter(args, Some(obs_type), None);

    let limit = args.limit.unwrap_or(10) as usize;
    let fetch_limit = limit * 5;
    match memory_service
        .search_memories(query, Some(filter), fetch_limit)
        .await
    {
        Ok(results) => {
            let mut items: Vec<_> = results.iter().filter_map(mapper).collect();
            items.sort_by(|a, b| {
                b.get("created_at")
                    .and_then(Value::as_i64)
                    .cmp(&a.get("created_at").and_then(Value::as_i64))
            });
            items.truncate(limit);
            crate::formatter::ResponseFormatter::json_success(&serde_json::json!({
                "count": items.len(),
                result_key: items,
            }))
        }
        Err(e) => Ok(crate::error_mapping::to_contextual_tool_error(e)),
    }
}

pub(super) fn build_memory_filter(
    args: &MemoryArgs,
    obs_type: Option<ObservationType>,
    tags: Option<Vec<String>>,
) -> MemoryFilter {
    MemoryFilter {
        id: None,
        project_id: args.project_id.clone(),
        tags,
        r#type: obs_type,
        session_id: args
            .session_id
            .clone()
            .map(|id| compute_stable_id_hash("session", id.as_str())),
        parent_session_id: args.parent_session_id.clone(),
        repo_id: args.repo_id.clone(),
        time_range: None,
        branch: None,
        commit: None,
    }
}
