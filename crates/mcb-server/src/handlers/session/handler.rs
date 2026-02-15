//! Session handler implementation.

use std::sync::Arc;

use mcb_domain::ports::services::AgentSessionServiceInterface;
use mcb_domain::ports::services::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use super::{create, get, list, summarize, update};
use crate::args::{SessionAction, SessionArgs};
use crate::handlers::helpers::resolve_org_id;

/// Handler for agent session MCP tool operations.
///
/// Supports creating, updating, listing, and summarizing agent sessions.
#[derive(Clone)]
pub struct SessionHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SessionHandler {
    /// Create a new `SessionHandler`.
    pub fn new(
        agent_service: Arc<dyn AgentSessionServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self {
            agent_service,
            memory_service,
        }
    }

    /// Handle a session tool request.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<SessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        let _org_id = resolve_org_id(args.org_id.as_deref());

        match args.action {
            SessionAction::Create => create::create_session(&self.agent_service, &args).await,
            SessionAction::Get => get::get_session(&self.agent_service, &args).await,
            SessionAction::Update => update::update_session(&self.agent_service, &args).await,
            SessionAction::List => list::list_sessions(&self.agent_service, &args).await,
            SessionAction::Summarize => {
                summarize::summarize_session(&self.memory_service, &args).await
            }
        }
    }
}
