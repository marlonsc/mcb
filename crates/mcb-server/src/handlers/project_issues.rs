//! Handlers for project issue MCP tools.

use crate::args::{
    ProjectAddDependencyArgs, ProjectCreateIssueArgs, ProjectListIssuesArgs, ProjectUpdateIssueArgs,
};
use mcb_domain::entities::project::{
    DependencyType, IssueStatus, IssueType, ProjectDependency, ProjectIssue,
};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

/// Handler for the MCP `project_create_issue` tool.
#[derive(Default)]
pub struct ProjectCreateIssueHandler;

impl ProjectCreateIssueHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectCreateIssueArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _issue: Option<ProjectIssue> = None;
        let _issue_type: Option<IssueType> = args.issue_type.parse().ok();
        let _status: Option<IssueStatus> = args.status.parse().ok();

        Err(McpError::internal_error("Not implemented", None))
    }
}

/// Handler for the MCP `project_update_issue` tool.
#[derive(Default)]
pub struct ProjectUpdateIssueHandler;

impl ProjectUpdateIssueHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectUpdateIssueArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _issue: Option<ProjectIssue> = None;
        let _issue_type: Option<IssueType> = args
            .issue_type
            .as_ref()
            .and_then(|value| value.parse().ok());
        let _status: Option<IssueStatus> =
            args.status.as_ref().and_then(|value| value.parse().ok());

        Err(McpError::internal_error("Not implemented", None))
    }
}

/// Handler for the MCP `project_list_issues` tool.
#[derive(Default)]
pub struct ProjectListIssuesHandler;

impl ProjectListIssuesHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectListIssuesArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _issue: Option<ProjectIssue> = None;
        let _status: Option<IssueStatus> =
            args.status.as_ref().and_then(|value| value.parse().ok());

        Err(McpError::internal_error("Not implemented", None))
    }
}

/// Handler for the MCP `project_add_dependency` tool.
#[derive(Default)]
pub struct ProjectAddDependencyHandler;

impl ProjectAddDependencyHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectAddDependencyArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _dependency: Option<ProjectDependency> = None;
        let _dependency_type: Option<DependencyType> = args.dependency_type.parse().ok();

        Err(McpError::internal_error("Not implemented", None))
    }
}
