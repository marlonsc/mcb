use crate::args::ListAgentSessionsArgs;
use mcb_application::ports::services::{AgentSessionQuery, AgentSessionServiceInterface};
use mcb_domain::entities::agent::{AgentSessionStatus, AgentType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

pub struct ListAgentSessionsHandler {
    service: Arc<dyn AgentSessionServiceInterface>,
}

#[derive(Serialize)]
struct SessionSummary {
    id: String,
    agent_type: String,
    status: String,
    started_at: i64,
    duration_ms: Option<i64>,
}

#[derive(Serialize)]
struct ListResult {
    sessions: Vec<SessionSummary>,
    count: usize,
}

impl ListAgentSessionsHandler {
    pub fn new(service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ListAgentSessionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let agent_type: Option<AgentType> = args
            .agent_type
            .map(|s| s.parse())
            .transpose()
            .map_err(|_| McpError::invalid_params("Invalid agent_type", None))?;

        let status: Option<AgentSessionStatus> = args
            .status
            .map(|s| s.parse())
            .transpose()
            .map_err(|_| McpError::invalid_params("Invalid status", None))?;

        let query = AgentSessionQuery {
            session_summary_id: args.session_summary_id,
            parent_session_id: args.parent_session_id,
            agent_type,
            status,
            limit: Some(args.limit),
        };

        match self.service.list_sessions(query).await {
            Ok(sessions) => {
                let summaries: Vec<SessionSummary> = sessions
                    .into_iter()
                    .map(|s| SessionSummary {
                        id: s.id,
                        agent_type: s.agent_type.as_str().to_string(),
                        status: s.status.as_str().to_string(),
                        started_at: s.started_at,
                        duration_ms: s.duration_ms,
                    })
                    .collect();

                let count = summaries.len();
                let result = ListResult {
                    sessions: summaries,
                    count,
                };

                let json = serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| String::from("Failed to serialize result"));

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to list agent sessions",
            )])),
        }
    }
}
