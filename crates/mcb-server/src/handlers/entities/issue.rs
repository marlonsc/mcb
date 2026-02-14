//! Issue entity CRUD handler implementation.

use std::sync::Arc;

use mcb_domain::entities::issue::{IssueComment, IssueLabel, IssueLabelAssignment};
use mcb_domain::entities::project::ProjectIssue;
use mcb_domain::ports::repositories::IssueEntityRepository;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use crate::handlers::helpers::{
    map_opaque_error, ok_json, ok_text, require_data, require_id, resolve_org_id,
};

/// Handler for the consolidated `issue_entity` MCP tool.
pub struct IssueEntityHandler {
    repo: Arc<dyn IssueEntityRepository>,
}

impl IssueEntityHandler {
    /// Create a new issue entity handler backed by a repository implementation.
    pub fn new(repo: Arc<dyn IssueEntityRepository>) -> Self {
        Self { repo }
    }

    /// Route an incoming `issue_entity` tool call to the appropriate CRUD operation.
    /// # Architecture Violation (KISS005)
    /// Function length (103 lines) exceeds the 50-line limit.
    ///
    // TODO(KISS005): Break 'handle' into smaller, focused functions.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IssueEntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let org_id = resolve_org_id(args.org_id.as_deref());

        crate::entity_crud_dispatch! {
            action = args.action,
            resource = args.resource,
            {
            (IssueEntityAction::Create, IssueEntityResource::Issue) => {
                let mut issue: ProjectIssue = require_data(args.data, "data required for create")?;
                issue.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_issue(&issue).await)?;
                ok_json(&issue)
            }
            (IssueEntityAction::Get, IssueEntityResource::Issue) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_issue(org_id.as_str(), &id).await)?)
            }
            (IssueEntityAction::List, IssueEntityResource::Issue) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                ok_json(&map_opaque_error(self.repo.list_issues(org_id.as_str(), project_id).await)?)
            }
            (IssueEntityAction::Update, IssueEntityResource::Issue) => {
                let mut issue: ProjectIssue = require_data(args.data, "data required for update")?;
                issue.org_id = org_id.to_string();
                map_opaque_error(self.repo.update_issue(&issue).await)?;
                ok_text("updated")
            }
            (IssueEntityAction::Delete, IssueEntityResource::Issue) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_issue(org_id.as_str(), &id).await)?;
                ok_text("deleted")
            }
            (IssueEntityAction::Create, IssueEntityResource::Comment) => {
                let comment: IssueComment = require_data(args.data, "data required")?;
                map_opaque_error(self.repo.create_comment(&comment).await)?;
                ok_json(&comment)
            }
            (IssueEntityAction::Get, IssueEntityResource::Comment) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_comment(&id).await)?)
            }
            (IssueEntityAction::List, IssueEntityResource::Comment) => {
                let issue_id = args
                    .issue_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("issue_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_comments_by_issue(issue_id).await)?)
            }
            (IssueEntityAction::Delete, IssueEntityResource::Comment) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_comment(&id).await)?;
                ok_text("deleted")
            }
            (IssueEntityAction::Create, IssueEntityResource::Label) => {
                let mut label: IssueLabel = require_data(args.data, "data required")?;
                label.org_id = org_id.to_string();
                map_opaque_error(self.repo.create_label(&label).await)?;
                ok_json(&label)
            }
            (IssueEntityAction::Get, IssueEntityResource::Label) => {
                let id = require_id(&args.id)?;
                ok_json(&map_opaque_error(self.repo.get_label(&id).await)?)
            }
            (IssueEntityAction::List, IssueEntityResource::Label) => {
                let project_id = args.project_id.as_deref().ok_or_else(|| {
                    McpError::invalid_params("project_id required for list", None)
                })?;
                ok_json(&map_opaque_error(self.repo.list_labels(org_id.as_str(), project_id).await)?)
            }
            (IssueEntityAction::Delete, IssueEntityResource::Label) => {
                let id = require_id(&args.id)?;
                map_opaque_error(self.repo.delete_label(&id).await)?;
                ok_text("deleted")
            }
            (IssueEntityAction::Create, IssueEntityResource::LabelAssignment) => {
                // TODO(ERR001): Missing error context.
                let assignment: IssueLabelAssignment = require_data(args.data, "data required")?;
                // TODO(ERR001): Missing error context.
                map_opaque_error(self.repo.assign_label(&assignment).await)?;
                ok_text("assigned")
            }
            (IssueEntityAction::Delete, IssueEntityResource::LabelAssignment) => {
                let issue_id = args
                    .issue_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("issue_id required", None))?;
                let label_id = args
                    .label_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("label_id required", None))?;
                // TODO(ERR001): Missing error context.
                map_opaque_error(self.repo.unassign_label(issue_id, label_id).await)?;
                ok_text("unassigned")
            }
            (IssueEntityAction::List, IssueEntityResource::LabelAssignment) => {
                let issue_id = args
                    .issue_id
                    .as_deref()
                    .ok_or_else(|| McpError::invalid_params("issue_id required", None))?;
                ok_json(&map_opaque_error(self.repo.list_labels_for_issue(issue_id).await)?)
            }
            }
        }
    }
}
