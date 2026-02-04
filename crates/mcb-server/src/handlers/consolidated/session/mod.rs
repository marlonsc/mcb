//! Session handler for agent session management.
//!
//! This module provides a unified handler for agent session MCP tool operations.

mod create;
mod get;
mod helpers;
mod list;
mod summarize;
mod update;

use crate::args::{SessionAction, SessionArgs};
use mcb_application::ports::MemoryServiceInterface;
use mcb_application::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use std::sync::Arc;
use validator::Validate;

pub use helpers::SessionHelpers;

/// Handler for agent session MCP tool operations.
///
/// Supports creating, updating, listing, and summarizing agent sessions.
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

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

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
