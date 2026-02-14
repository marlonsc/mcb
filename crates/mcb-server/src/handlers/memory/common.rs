use mcb_domain::entities::memory::{ExecutionMetadata, ObservationMetadata, QualityGateResult};
use std::sync::Arc;

use crate::handlers::helpers::hash_id;
use mcb_domain::{
    entities::memory::{MemoryFilter, MemorySearchResult, ObservationType},
    ports::services::MemoryServiceInterface,
};
use mcb_infrastructure::project::context_resolver::capture_vcs_context;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::args::MemoryArgs;
use crate::handlers::helpers::{OriginPayloadFields, resolve_origin_context};
pub(super) use crate::handlers::helpers::{opt_str, require_data_map, require_str, str_vec};

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
    let vcs_context = capture_vcs_context();
    let canonical_session_id = args
        .session_id
        .clone()
        .map(|id| hash_id("session", id.as_str()));
    let parent_session_hash = args
        .parent_session_id
        .clone()
        .map(|id| hash_id("parent_session", id.as_str()));

    let payload = OriginPayloadFields::extract(data);
    let mut input = payload.to_input();
    input.org_id = args.org_id.as_deref();
    input.project_id_args = args.project_id.as_deref();
    input.execution_from_args = opts.execution_from_args;
    input.execution_from_data = opts.execution_from_data;
    input.tool_name_args = Some("memory");
    input.repo_id_args = args.repo_id.as_deref();
    input.file_path_payload = opts.file_path_payload;
    input.require_project_id = true;
    input.timestamp = opts.timestamp;
    let mut origin_context = resolve_origin_context(input)?;

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
