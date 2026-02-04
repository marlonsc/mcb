use crate::args::{MemoryAction, MemoryArgs, MemoryResource};
use crate::formatter::ResponseFormatter;
use chrono::TimeZone;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{
    ExecutionMetadata, ExecutionType, MemoryFilter, ObservationMetadata, ObservationType,
    QualityGateResult, QualityGateStatus,
};
use mcb_domain::utils::vcs_context::VcsContext;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl MemoryHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
        data.as_ref().and_then(|value| value.as_object())
    }

    fn get_str(data: &Map<String, Value>, key: &str) -> Option<String> {
        data.get(key)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    fn get_i64(data: &Map<String, Value>, key: &str) -> Option<i64> {
        data.get(key).and_then(|value| value.as_i64())
    }

    fn get_i32(data: &Map<String, Value>, key: &str) -> Option<i32> {
        data.get(key)
            .and_then(|value| value.as_i64())
            .and_then(|v| v.try_into().ok())
    }

    fn get_f32(data: &Map<String, Value>, key: &str) -> Option<f32> {
        data.get(key)
            .and_then(|value| value.as_f64())
            .map(|v| v as f32)
    }

    fn get_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
        data.get(key).and_then(|value| value.as_bool())
    }

    fn get_string_list(data: &Map<String, Value>, key: &str) -> Vec<String> {
        data.get(key)
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_required_str(data: &Map<String, Value>, key: &str) -> Result<String, CallToolResult> {
        Self::get_str(data, key).ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
    }

    fn parse_observation_type(value: &str) -> Result<ObservationType, CallToolResult> {
        match value.to_lowercase().as_str() {
            "code" => Ok(ObservationType::Code),
            "decision" => Ok(ObservationType::Decision),
            "context" => Ok(ObservationType::Context),
            "error" => Ok(ObservationType::Error),
            "summary" => Ok(ObservationType::Summary),
            "execution" => Ok(ObservationType::Execution),
            "quality_gate" => Ok(ObservationType::QualityGate),
            _ => Err(CallToolResult::error(vec![Content::text(format!(
                "Unknown observation type: {value}"
            ))])),
        }
    }

    fn parse_execution_type(value: &str) -> Result<ExecutionType, CallToolResult> {
        value.parse().map_err(|_| {
            CallToolResult::error(vec![Content::text(format!(
                "Unknown execution type: {value}"
            ))])
        })
    }

    fn parse_quality_gate_status(value: &str) -> Result<QualityGateStatus, CallToolResult> {
        value.parse().map_err(|_| {
            CallToolResult::error(vec![Content::text(format!(
                "Unknown quality gate status: {value}"
            ))])
        })
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            MemoryAction::Store => match args.resource {
                MemoryResource::Observation => {
                    let data = match Self::json_map(&args.data) {
                        Some(data) => data,
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Missing data payload for observation store",
                            )]));
                        }
                    };
                    let content = match Self::get_required_str(data, "content") {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let observation_type_str =
                        match Self::get_required_str(data, "observation_type") {
                            Ok(v) => v,
                            Err(error_result) => return Ok(error_result),
                        };
                    let observation_type = match Self::parse_observation_type(&observation_type_str)
                    {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let tags = Self::get_string_list(data, "tags");
                    let vcs_context = VcsContext::capture();
                    let metadata = ObservationMetadata {
                        id: Uuid::new_v4().to_string(),
                        session_id: Self::get_str(data, "session_id")
                            .or_else(|| args.session_id.clone()),
                        repo_id: Self::get_str(data, "repo_id")
                            .or_else(|| args.repo_id.clone())
                            .or_else(|| vcs_context.repo_id.clone()),
                        file_path: Self::get_str(data, "file_path"),
                        branch: Self::get_str(data, "branch")
                            .or_else(|| vcs_context.branch.clone()),
                        commit: Self::get_str(data, "commit")
                            .or_else(|| vcs_context.commit.clone()),
                        execution: None,
                        quality_gate: None,
                    };
                    match self
                        .memory_service
                        .store_observation(content, observation_type, tags, metadata)
                        .await
                    {
                        Ok((observation_id, deduplicated)) => {
                            ResponseFormatter::json_success(&serde_json::json!({
                                "observation_id": observation_id,
                                "deduplicated": deduplicated,
                            }))
                        }
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to store observation: {}",
                            e
                        ))])),
                    }
                }
                MemoryResource::Execution => {
                    let data = match Self::json_map(&args.data) {
                        Some(data) => data,
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Missing data payload for execution store",
                            )]));
                        }
                    };
                    let command = match Self::get_required_str(data, "command") {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let exit_code = match Self::get_i32(data, "exit_code").ok_or_else(|| {
                        CallToolResult::error(vec![Content::text(
                            "Missing required field: exit_code",
                        )])
                    }) {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let duration_ms = match Self::get_i64(data, "duration_ms").ok_or_else(|| {
                        CallToolResult::error(vec![Content::text(
                            "Missing required field: duration_ms",
                        )])
                    }) {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let success = match Self::get_bool(data, "success").ok_or_else(|| {
                        CallToolResult::error(vec![Content::text(
                            "Missing required field: success",
                        )])
                    }) {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let execution_type_str = match Self::get_required_str(data, "execution_type") {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let execution_type = match Self::parse_execution_type(&execution_type_str) {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let metadata = ExecutionMetadata {
                        id: Uuid::new_v4().to_string(),
                        command: command.clone(),
                        exit_code: Some(exit_code),
                        duration_ms: Some(duration_ms),
                        success,
                        execution_type,
                        coverage: Self::get_f32(data, "coverage"),
                        files_affected: Self::get_string_list(data, "files_affected"),
                        output_summary: Self::get_str(data, "output_summary"),
                        warnings_count: Self::get_i32(data, "warnings_count"),
                        errors_count: Self::get_i32(data, "errors_count"),
                    };
                    let vcs_context = VcsContext::capture();
                    let content = format!(
                        "Execution: {} (exit_code={}, success={})",
                        command, exit_code, success
                    );
                    let tags = vec![
                        "execution".to_string(),
                        metadata.execution_type.as_str().to_string(),
                        if success { "success" } else { "failure" }.to_string(),
                    ];
                    let obs_metadata = ObservationMetadata {
                        id: Uuid::new_v4().to_string(),
                        session_id: Self::get_str(data, "session_id")
                            .or_else(|| args.session_id.clone()),
                        repo_id: Self::get_str(data, "repo_id")
                            .or_else(|| args.repo_id.clone())
                            .or_else(|| vcs_context.repo_id.clone()),
                        file_path: None,
                        branch: Self::get_str(data, "branch")
                            .or_else(|| vcs_context.branch.clone()),
                        commit: Self::get_str(data, "commit")
                            .or_else(|| vcs_context.commit.clone()),
                        execution: Some(metadata),
                        quality_gate: None,
                    };
                    match self
                        .memory_service
                        .store_observation(content, ObservationType::Execution, tags, obs_metadata)
                        .await
                    {
                        Ok((observation_id, deduplicated)) => {
                            ResponseFormatter::json_success(&serde_json::json!({
                                "observation_id": observation_id,
                                "deduplicated": deduplicated,
                            }))
                        }
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to store execution: {}",
                            e
                        ))])),
                    }
                }
                MemoryResource::QualityGate => {
                    let data = match Self::json_map(&args.data) {
                        Some(data) => data,
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Missing data payload for quality gate store",
                            )]));
                        }
                    };
                    let gate_name = match Self::get_required_str(data, "gate_name") {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let status_str = match Self::get_required_str(data, "status") {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let status = match Self::parse_quality_gate_status(&status_str) {
                        Ok(v) => v,
                        Err(error_result) => return Ok(error_result),
                    };
                    let timestamp = Self::get_i64(data, "timestamp")
                        .and_then(|ts| chrono::Utc.timestamp_opt(ts, 0).single())
                        .unwrap_or_else(chrono::Utc::now);
                    let quality_gate = QualityGateResult {
                        id: Uuid::new_v4().to_string(),
                        gate_name: gate_name.clone(),
                        status,
                        message: Self::get_str(data, "message"),
                        timestamp,
                        execution_id: Self::get_str(data, "execution_id"),
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
                    let obs_metadata = ObservationMetadata {
                        id: Uuid::new_v4().to_string(),
                        session_id: Self::get_str(data, "session_id")
                            .or_else(|| args.session_id.clone()),
                        repo_id: Self::get_str(data, "repo_id")
                            .or_else(|| args.repo_id.clone())
                            .or_else(|| vcs_context.repo_id.clone()),
                        file_path: None,
                        branch: Self::get_str(data, "branch")
                            .or_else(|| vcs_context.branch.clone()),
                        commit: Self::get_str(data, "commit")
                            .or_else(|| vcs_context.commit.clone()),
                        execution: None,
                        quality_gate: Some(quality_gate),
                    };
                    match self
                        .memory_service
                        .store_observation(
                            content,
                            ObservationType::QualityGate,
                            tags,
                            obs_metadata,
                        )
                        .await
                    {
                        Ok((observation_id, deduplicated)) => {
                            ResponseFormatter::json_success(&serde_json::json!({
                                "observation_id": observation_id,
                                "deduplicated": deduplicated,
                            }))
                        }
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to store quality gate: {}",
                            e
                        ))])),
                    }
                }
                MemoryResource::ErrorPattern => Ok(CallToolResult::error(vec![Content::text(
                    "Error pattern memory is not implemented yet",
                )])),
                MemoryResource::Session => {
                    let data = match Self::json_map(&args.data) {
                        Some(data) => data,
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Missing data payload for session summary",
                            )]));
                        }
                    };
                    let session_id = args
                        .session_id
                        .clone()
                        .or_else(|| Self::get_str(data, "session_id"));
                    let session_id = match session_id {
                        Some(value) => value,
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Missing session_id for session summary",
                            )]));
                        }
                    };
                    let topics = Self::get_string_list(data, "topics");
                    let decisions = Self::get_string_list(data, "decisions");
                    let next_steps = Self::get_string_list(data, "next_steps");
                    let key_files = Self::get_string_list(data, "key_files");
                    match self
                        .memory_service
                        .create_session_summary(
                            session_id.clone(),
                            topics,
                            decisions,
                            next_steps,
                            key_files,
                        )
                        .await
                    {
                        Ok(summary_id) => ResponseFormatter::json_success(&serde_json::json!({
                            "summary_id": summary_id,
                            "session_id": session_id,
                        })),
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to create session summary: {}",
                            e
                        ))])),
                    }
                }
            },
            MemoryAction::Get => match args.resource {
                MemoryResource::Observation => {
                    let ids = args.ids.clone().unwrap_or_default();
                    if ids.is_empty() {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing observation ids",
                        )]));
                    }
                    match self.memory_service.get_observations_by_ids(&ids).await {
                        Ok(observations) => {
                            let observations: Vec<_> = observations
                                .into_iter()
                                .map(|obs| {
                                    serde_json::json!({
                                        "id": obs.id,
                                        "content": obs.content,
                                        "observation_type": obs.observation_type.as_str(),
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
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to get observations: {}",
                            e
                        ))])),
                    }
                }
                MemoryResource::Execution => {
                    let filter = MemoryFilter {
                        id: None,
                        tags: None,
                        observation_type: Some(ObservationType::Execution),
                        session_id: args.session_id.clone(),
                        repo_id: args.repo_id.clone(),
                        time_range: None,
                        branch: None,
                        commit: None,
                    };
                    let query = "execution".to_string();
                    let limit = args.limit.unwrap_or(10) as usize;
                    let fetch_limit = limit * 5;
                    match self
                        .memory_service
                        .search_memories(&query, Some(filter), fetch_limit)
                        .await
                    {
                        Ok(results) => {
                            let mut executions: Vec<_> = results
                                .into_iter()
                                .filter_map(|result| {
                                    let execution = result.observation.metadata.execution?;
                                    Some(serde_json::json!({
                                        "observation_id": result.observation.id,
                                        "command": execution.command,
                                        "exit_code": execution.exit_code,
                                        "duration_ms": execution.duration_ms,
                                        "success": execution.success,
                                        "execution_type": execution.execution_type.as_str(),
                                        "coverage": execution.coverage,
                                        "files_affected": execution.files_affected,
                                        "output_summary": execution.output_summary,
                                        "warnings_count": execution.warnings_count,
                                        "errors_count": execution.errors_count,
                                        "created_at": result.observation.created_at,
                                    }))
                                })
                                .collect();
                            executions.sort_by(|a, b| {
                                b.get("created_at")
                                    .and_then(|v| v.as_i64())
                                    .cmp(&a.get("created_at").and_then(|v| v.as_i64()))
                            });
                            executions.truncate(limit);
                            ResponseFormatter::json_success(&serde_json::json!({
                                "count": executions.len(),
                                "executions": executions,
                            }))
                        }
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to get executions: {}",
                            e
                        ))])),
                    }
                }
                MemoryResource::QualityGate => {
                    let filter = MemoryFilter {
                        id: None,
                        tags: None,
                        observation_type: Some(ObservationType::QualityGate),
                        session_id: args.session_id.clone(),
                        repo_id: args.repo_id.clone(),
                        time_range: None,
                        branch: None,
                        commit: None,
                    };
                    let query = "quality gate".to_string();
                    let limit = args.limit.unwrap_or(10) as usize;
                    let fetch_limit = limit * 5;
                    match self
                        .memory_service
                        .search_memories(&query, Some(filter), fetch_limit)
                        .await
                    {
                        Ok(results) => {
                            let mut gates: Vec<_> = results
                                .into_iter()
                                .filter_map(|result| {
                                    let gate = result.observation.metadata.quality_gate?;
                                    Some(serde_json::json!({
                                        "observation_id": result.observation.id,
                                        "gate_name": gate.gate_name,
                                        "status": gate.status.as_str(),
                                        "message": gate.message,
                                        "timestamp": gate.timestamp,
                                        "execution_id": gate.execution_id,
                                        "created_at": result.observation.created_at,
                                    }))
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
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to get quality gates: {}",
                            e
                        ))])),
                    }
                }
                MemoryResource::ErrorPattern => Ok(CallToolResult::error(vec![Content::text(
                    "Error pattern memory is not implemented yet",
                )])),
                MemoryResource::Session => {
                    let session_id = match args.session_id.as_ref() {
                        Some(value) => value,
                        None => {
                            return Ok(CallToolResult::error(vec![Content::text(
                                "Missing session_id",
                            )]));
                        }
                    };
                    match self.memory_service.get_session_summary(session_id).await {
                        Ok(Some(summary)) => ResponseFormatter::json_success(&serde_json::json!({
                            "session_id": summary.session_id,
                            "topics": summary.topics,
                            "decisions": summary.decisions,
                            "next_steps": summary.next_steps,
                            "key_files": summary.key_files,
                            "created_at": summary.created_at,
                        })),
                        Ok(None) => Ok(CallToolResult::error(vec![Content::text(
                            "Session summary not found",
                        )])),
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to get session summary: {}",
                            e
                        ))])),
                    }
                }
            },
            MemoryAction::List => match args.resource {
                MemoryResource::Observation => {
                    let filter = MemoryFilter {
                        id: None,
                        tags: args.tags.clone(),
                        observation_type: None,
                        session_id: args.session_id.clone(),
                        repo_id: args.repo_id.clone(),
                        time_range: None,
                        branch: None,
                        commit: None,
                    };
                    let limit = args.limit.unwrap_or(10) as usize;
                    let query = args.query.clone().unwrap_or_default();
                    match self
                        .memory_service
                        .memory_search(&query, Some(filter), limit)
                        .await
                    {
                        Ok(results) => {
                            let items: Vec<_> = results
                                .into_iter()
                                .map(|item| {
                                    serde_json::json!({
                                        "id": item.id,
                                        "observation_type": item.observation_type.as_str(),
                                        "relevance_score": item.relevance_score,
                                        "tags": item.tags,
                                        "content_preview": item.content_preview,
                                        "session_id": item.session_id,
                                        "repo_id": item.repo_id,
                                        "file_path": item.file_path,
                                        "created_at": item.created_at,
                                    })
                                })
                                .collect();
                            ResponseFormatter::json_success(&serde_json::json!({
                                "query": query,
                                "count": items.len(),
                                "results": items,
                                "hint": "Use memory action=timeline or memory action=get for details",
                            }))
                        }
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to list memories: {}",
                            e
                        ))])),
                    }
                }
                _ => Ok(CallToolResult::error(vec![Content::text(
                    "List action is only supported for observation resource",
                )])),
            },
            MemoryAction::Timeline => {
                let anchor_id = if let Some(anchor_id) = args.anchor_id.clone() {
                    anchor_id
                } else if let Some(query) = args.query.clone() {
                    let results = self
                        .memory_service
                        .search_memories(&query, None, 1)
                        .await
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to find anchor from query: {e}"),
                                None,
                            )
                        })?;
                    if let Some(first) = results.first() {
                        first.observation.id.clone()
                    } else {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "No anchor observation found",
                        )]));
                    }
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Missing anchor_id or query for timeline",
                    )]));
                };
                let filter = MemoryFilter {
                    id: None,
                    tags: None,
                    observation_type: None,
                    session_id: args.session_id.clone(),
                    repo_id: args.repo_id.clone(),
                    time_range: None,
                    branch: None,
                    commit: None,
                };
                let depth_before = args.depth_before.unwrap_or(5);
                let depth_after = args.depth_after.unwrap_or(5);
                match self
                    .memory_service
                    .get_timeline(&anchor_id, depth_before, depth_after, Some(filter))
                    .await
                {
                    Ok(timeline) => {
                        let items: Vec<_> = timeline
                            .into_iter()
                            .map(|observation| {
                                serde_json::json!({
                                    "observation_id": observation.id,
                                    "content": observation.content,
                                    "observation_type": observation.observation_type.as_str(),
                                    "created_at": observation.created_at,
                                })
                            })
                            .collect();
                        ResponseFormatter::json_success(&serde_json::json!({
                            "anchor_id": anchor_id,
                            "count": items.len(),
                            "timeline": items,
                        }))
                    }
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to get timeline: {}",
                        e
                    ))])),
                }
            }
            MemoryAction::Inject => {
                let filter = MemoryFilter {
                    id: None,
                    tags: None,
                    observation_type: None,
                    session_id: args.session_id.clone(),
                    repo_id: args.repo_id.clone(),
                    time_range: None,
                    branch: None,
                    commit: None,
                };
                let limit = args.limit.unwrap_or(10) as usize;
                let max_tokens = args.max_tokens.unwrap_or(2000);
                let vcs_context = VcsContext::capture();
                match self
                    .memory_service
                    .search_memories("", Some(filter), limit)
                    .await
                {
                    Ok(results) => {
                        let mut context = String::new();
                        let mut observation_ids = Vec::new();
                        let max_chars = max_tokens * 4;
                        for result in results {
                            observation_ids.push(result.observation.id.clone());
                            let entry = format!(
                                "[{}] {}: {}\n\n",
                                result.observation.observation_type.as_str().to_uppercase(),
                                result.observation.id,
                                result.observation.content
                            );
                            if context.len() + entry.len() > max_chars {
                                break;
                            }
                            context.push_str(&entry);
                        }
                        ResponseFormatter::json_success(&serde_json::json!({
                            "session_id": args.session_id,
                            "observation_count": observation_ids.len(),
                            "observation_ids": observation_ids,
                            "context": context,
                            "estimated_tokens": context.len() / 4,
                            "vcs_context": {
                                "branch": vcs_context.branch,
                                "commit": vcs_context.commit,
                            }
                        }))
                    }
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to inject context: {}",
                        e
                    ))])),
                }
            }
        }
    }
}
