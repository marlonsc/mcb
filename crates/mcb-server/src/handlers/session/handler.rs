//! Session handler implementation.

use std::sync::Arc;

use mcb_application::services::RepositoryResolver;
use mcb_domain::ports::services::AgentSessionServiceInterface;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::OrgContext;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use super::{create, get, list, summarize, update};
use crate::args::{SessionAction, SessionArgs};

/// Handler for agent session MCP tool operations.
///
/// Supports creating, updating, listing, and summarizing agent sessions.
#[derive(Clone)]
pub struct SessionHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
    resolver: Arc<RepositoryResolver>,
}

impl SessionHandler {
    /// Create a new SessionHandler.
    pub fn new(
        agent_service: Arc<dyn AgentSessionServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
        resolver: Arc<RepositoryResolver>,
    ) -> Self {
        Self {
            agent_service,
            memory_service,
            resolver,
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

        let org_ctx = OrgContext::current();
        let org_id = org_ctx.id_str();
        let project_id = self.resolver.resolve_project_id(org_id).await;

        match args.action {
            SessionAction::Create => {
                create::create_session(&self.agent_service, &args, &project_id).await
            }
            SessionAction::Get => get::get_session(&self.agent_service, &args).await,
            SessionAction::Update => update::update_session(&self.agent_service, &args).await,
            SessionAction::List => {
                list::list_sessions(&self.agent_service, &args, &project_id).await
            }
            SessionAction::Summarize => {
                summarize::summarize_session(&self.memory_service, &args).await
            }
        }
    }
}
