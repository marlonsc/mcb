use crate::args::StoreDelegationArgs;
use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_domain::entities::agent::Delegation;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::Validate;

pub struct StoreDelegationHandler {
    service: Arc<dyn AgentSessionServiceInterface>,
}

#[derive(Serialize)]
struct StoreResult {
    delegation_id: String,
    parent_session_id: String,
    child_session_id: String,
}

impl StoreDelegationHandler {
    pub fn new(service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<StoreDelegationArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let delegation_id = format!("del_{}", uuid::Uuid::new_v4());

        let delegation = Delegation {
            id: delegation_id.clone(),
            parent_session_id: args.parent_session_id.clone(),
            child_session_id: args.child_session_id.clone(),
            prompt: args.prompt,
            prompt_embedding_id: None,
            result: args.result,
            success: args.success,
            created_at: now,
            completed_at: if args.success { Some(now) } else { None },
            duration_ms: args.duration_ms,
        };

        match self.service.store_delegation(delegation).await {
            Ok(_) => {
                let result = StoreResult {
                    delegation_id,
                    parent_session_id: args.parent_session_id,
                    child_session_id: args.child_session_id,
                };

                let json = serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| String::from("Failed to serialize result"));

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to store delegation",
            )])),
        }
    }
}
