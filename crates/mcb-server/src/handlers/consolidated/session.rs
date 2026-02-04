use crate::args::{SessionAction, SessionArgs};
use crate::formatter::ResponseFormatter;
use mcb_application::ports::MemoryServiceInterface;
use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_application::services::AgentSessionQuery;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct SessionHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SessionHandler {
    pub fn new(
        agent_service: Arc<dyn AgentSessionServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self {
            agent_service,
            memory_service,
        }
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

    fn parse_agent_type(value: &str) -> Result<AgentType, McpError> {
        value
            .parse()
            .map_err(|_| McpError::invalid_params("Invalid agent_type", None))
    }

    fn parse_status(value: &str) -> Result<AgentSessionStatus, McpError> {
        value
            .parse()
            .map_err(|_| McpError::invalid_params("Invalid status", None))
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            SessionAction::Create => {
                let agent_type = match args.agent_type.as_ref() {
                    Some(value) => Self::parse_agent_type(value)?,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing agent_type for create",
                        )]));
                    }
                };
                let data = match Self::json_map(&args.data) {
                    Some(data) => data,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing data payload for create",
                        )]));
                    }
                };
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let session_id = format!("agent_{}", Uuid::new_v4());
                let session_summary_id = match Self::get_required_str(data, "session_summary_id") {
                    Ok(v) => v,
                    Err(error_result) => return Ok(error_result),
                };
                let model = match Self::get_required_str(data, "model") {
                    Ok(v) => v,
                    Err(error_result) => return Ok(error_result),
                };
                let session = AgentSession {
                    id: session_id.clone(),
                    session_summary_id,
                    agent_type: agent_type.clone(),
                    model,
                    parent_session_id: Self::get_str(data, "parent_session_id"),
                    started_at: now,
                    ended_at: None,
                    duration_ms: None,
                    status: AgentSessionStatus::Active,
                    prompt_summary: Self::get_str(data, "prompt_summary"),
                    result_summary: None,
                    token_count: None,
                    tool_calls_count: None,
                    delegations_count: None,
                };
                match self.agent_service.create_session(session).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "session_id": id,
                        "agent_type": agent_type.as_str(),
                        "status": "active",
                    })),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to create agent session",
                    )])),
                }
            }
            SessionAction::Get => {
                let session_id = match args.session_id.as_ref() {
                    Some(id) => id,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing session_id",
                        )]));
                    }
                };
                match self.agent_service.get_session(session_id).await {
                    Ok(Some(session)) => ResponseFormatter::json_success(&serde_json::json!({
                        "id": session.id,
                        "session_summary_id": session.session_summary_id,
                        "agent_type": session.agent_type.as_str(),
                        "model": session.model,
                        "parent_session_id": session.parent_session_id,
                        "started_at": session.started_at,
                        "ended_at": session.ended_at,
                        "duration_ms": session.duration_ms,
                        "status": session.status.as_str(),
                        "prompt_summary": session.prompt_summary,
                        "result_summary": session.result_summary,
                        "token_count": session.token_count,
                        "tool_calls_count": session.tool_calls_count,
                        "delegations_count": session.delegations_count,
                    })),
                    Ok(None) => Ok(CallToolResult::error(vec![Content::text(
                        "Agent session not found",
                    )])),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to get agent session",
                    )])),
                }
            }
            SessionAction::Update => {
                let session_id = match args.session_id.as_ref() {
                    Some(id) => id,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing session_id",
                        )]));
                    }
                };
                let data = Self::json_map(&args.data);
                let status = match args.status.as_ref() {
                    Some(status) => Some(Self::parse_status(status)?),
                    None => data
                        .and_then(|d| Self::get_str(d, "status"))
                        .map(|status| Self::parse_status(&status))
                        .transpose()?,
                };
                match self.agent_service.get_session(session_id).await {
                    Ok(Some(mut session)) => {
                        if let Some(status) = status {
                            session.status = status;
                        }
                        if let Some(data) = data {
                            session.result_summary =
                                Self::get_str(data, "result_summary").or(session.result_summary);
                            session.token_count =
                                Self::get_i64(data, "token_count").or(session.token_count);
                            session.tool_calls_count = Self::get_i64(data, "tool_calls_count")
                                .or(session.tool_calls_count);
                            session.delegations_count = Self::get_i64(data, "delegations_count")
                                .or(session.delegations_count);
                        }
                        let status_str = session.status.as_str().to_string();
                        match self.agent_service.update_session(session).await {
                            Ok(_) => ResponseFormatter::json_success(&serde_json::json!({
                                "id": session_id,
                                "status": &status_str,
                                "updated": true,
                            })),
                            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                                "Failed to update agent session",
                            )])),
                        }
                    }
                    Ok(None) => Ok(CallToolResult::error(vec![Content::text(
                        "Agent session not found",
                    )])),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to update agent session",
                    )])),
                }
            }
            SessionAction::List => {
                let query = AgentSessionQuery {
                    session_summary_id: None,
                    parent_session_id: None,
                    agent_type: args
                        .agent_type
                        .as_ref()
                        .map(|value| value.parse())
                        .transpose()
                        .map_err(|_| McpError::invalid_params("Invalid agent_type", None))?,
                    status: args
                        .status
                        .as_ref()
                        .map(|value| value.parse())
                        .transpose()
                        .map_err(|_| McpError::invalid_params("Invalid status", None))?,
                    limit: Some(args.limit.unwrap_or(10) as usize),
                };
                match self.agent_service.list_sessions(query).await {
                    Ok(sessions) => {
                        let items: Vec<_> = sessions
                            .iter()
                            .map(|session| {
                                serde_json::json!({
                                    "id": session.id,
                                    "agent_type": session.agent_type.as_str(),
                                    "status": session.status.as_str(),
                                    "started_at": session.started_at,
                                    "duration_ms": session.duration_ms,
                                })
                            })
                            .collect();
                        ResponseFormatter::json_success(&serde_json::json!({
                            "sessions": items,
                            "count": items.len(),
                        }))
                    }
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to list agent sessions",
                    )])),
                }
            }
            SessionAction::Summarize => {
                let session_id = match args.session_id.as_ref() {
                    Some(id) => id,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing session_id",
                        )]));
                    }
                };
                if let Some(data) = Self::json_map(&args.data) {
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
                } else {
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
            }
        }
    }
}
