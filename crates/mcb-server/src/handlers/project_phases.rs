//! Handlers for project phase MCP tools.

use crate::args::{ProjectCreatePhaseArgs, ProjectListPhasesArgs, ProjectUpdatePhaseArgs};
use mcb_domain::entities::project::{PhaseStatus, ProjectPhase};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

/// Handler for the MCP `project_create_phase` tool.
#[derive(Default)]
pub struct ProjectCreatePhaseHandler;

impl ProjectCreatePhaseHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectCreatePhaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _phase: Option<ProjectPhase> = None;
        let _status: Option<PhaseStatus> = args.status.parse().ok();

        Err(McpError::internal_error("Not implemented", None))
    }
}

/// Handler for the MCP `project_update_phase` tool.
#[derive(Default)]
pub struct ProjectUpdatePhaseHandler;

impl ProjectUpdatePhaseHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectUpdatePhaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _phase: Option<ProjectPhase> = None;
        let _status: Option<PhaseStatus> =
            args.status.as_ref().and_then(|value| value.parse().ok());

        Err(McpError::internal_error("Not implemented", None))
    }
}

/// Handler for the MCP `project_list_phases` tool.
#[derive(Default)]
pub struct ProjectListPhasesHandler;

impl ProjectListPhasesHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectListPhasesArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _phase: Option<ProjectPhase> = None;
        let _status: Option<PhaseStatus> =
            args.status.as_ref().and_then(|value| value.parse().ok());

        Err(McpError::internal_error("Not implemented", None))
    }
}
