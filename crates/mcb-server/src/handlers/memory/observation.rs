use std::sync::Arc;

use mcb_domain::entities::memory::ObservationMetadata;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::{compute_stable_id_hash, vcs_context::VcsContext};
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use uuid::Uuid;

use super::helpers::MemoryHelpers;
use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{OriginContextInput, resolve_origin_context};
use crate::utils::json::{self, JsonMapExt};

/// Stores a new semantic observation with the provided content, type, and tags.
#[tracing::instrument(skip_all)]
pub async fn store_observation(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match json::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for observation store",
            )]));
        }
    };
    let content = match data.required_string("content") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    // TODO(ORG002): Duplicate string literal "observation_type".
    // Consider using mcb_domain::schema::memory::COL_OBSERVATION_TYPE instead.
    let observation_type_str = match data.required_string("observation_type") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let observation_type = match MemoryHelpers::parse_observation_type(&observation_type_str) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let tags = data.string_list("tags");
    let vcs_context = VcsContext::capture();
    let arg_session_id = args
        .session_id
        .clone()
        .map(|id| compute_stable_id_hash("session", id.as_str()));
    let canonical_session_id = arg_session_id.clone();
    let parent_session_hash = args
        .parent_session_id
        .clone()
        .map(|id| compute_stable_id_hash("parent_session", id.as_str()));
    let payload_repo_id = data.string("repo_id");
    let payload_project_id = data.string("project_id");
    let payload_file_path = data.string("file_path");
    let payload_branch = data.string("branch");
    let payload_commit = data.string("commit");
    let payload_repo_path = data.string("repo_path");
    let payload_worktree_id = data.string("worktree_id");
    let payload_operator_id = data.string("operator_id");
    let payload_machine_id = data.string("machine_id");
    let payload_agent_program = data.string("agent_program");
    let payload_model_id = data.string("model_id");
    let payload_delegated = data.boolean("delegated");

    let mut origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        session_from_args: None,
        session_from_data: None,
        parent_session_from_args: None,
        parent_session_from_data: None,
        execution_from_args: None,
        execution_from_data: None,
        tool_name_args: Some("memory"),
        tool_name_payload: None,
        repo_id_args: args.repo_id.as_deref(),
        repo_id_payload: payload_repo_id.as_deref(),
        repo_path_args: None,
        repo_path_payload: payload_repo_path.as_deref(),
        worktree_id_args: None,
        worktree_id_payload: payload_worktree_id.as_deref(),
        file_path_args: None,
        file_path_payload: payload_file_path.as_deref(),
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
        timestamp: None,
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
        McpError::invalid_params("project_id is required for storing observation", None)
    })?;

    let metadata = ObservationMetadata {
        id: Uuid::new_v4().to_string(),
        session_id: canonical_session_id,
        repo_id: origin_context.repo_id.clone(),
        file_path: origin_context.file_path.clone(),
        branch: origin_context.branch.clone(),
        commit: origin_context.commit.clone(),
        execution: None,
        quality_gate: None,
        origin_context: Some(origin_context),
    };
    match memory_service
        .store_observation(project_id, content, observation_type, tags, metadata)
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}

/// Retrieves semantic observations by their unique identifiers.
#[tracing::instrument(skip_all)]
pub async fn get_observations(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let ids = args.ids.clone().unwrap_or_default();
    if ids.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Missing observation ids",
        )]));
    }
    match memory_service
        .get_observations_by_ids(
            &ids.iter()
                .map(|id| ObservationId::new(id.clone()))
                .collect::<Vec<_>>(),
        )
        .await
    {
        Ok(observations) => {
            let observations: Vec<_> = observations
                .into_iter()
                .map(|obs| {
                    serde_json::json!({
                        "id": obs.id,
                        "content": obs.content,
                        // TODO(ORG002): Duplicate string literal "observation_type".
                        "observation_type": obs.r#type.as_str(),
                        "tags": obs.tags,
                        "session_id": obs.metadata.session_id,
                        "repo_id": obs.metadata.repo_id,
                        "file_path": obs.metadata.file_path,
                        "branch": obs.metadata.branch,
                        "created_at": obs.created_at,
                        "content_hash": obs.content_hash,
                    })
                })
                .collect();
            ResponseFormatter::json_success(&serde_json::json!({
                "count": observations.len(),
                "observations": observations,
            }))
        }
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
