//! Handlers for project decision MCP tools.

use crate::args::{ProjectListDecisionsArgs, ProjectRecordDecisionArgs};
use mcb_domain::entities::project::ProjectDecision;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

/// Handler for the MCP `project_record_decision` tool.
#[derive(Default)]
pub struct ProjectRecordDecisionHandler;

impl ProjectRecordDecisionHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectRecordDecisionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _decision: Option<ProjectDecision> = None;

        Err(McpError::internal_error("Not implemented", None))
    }
}

/// Handler for the MCP `project_list_decisions` tool.
#[derive(Default)]
pub struct ProjectListDecisionsHandler;

impl ProjectListDecisionsHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectListDecisionsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _decision: Option<ProjectDecision> = None;

        Err(McpError::internal_error("Not implemented", None))
    }
}
