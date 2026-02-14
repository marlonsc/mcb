//! Unified entity CRUD handler implementation.

use std::sync::Arc;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData as McpError};

use crate::args::{
    EntityAction, EntityArgs, EntityResource, IssueEntityAction, IssueEntityArgs,
    IssueEntityResource, OrgEntityAction, OrgEntityArgs, OrgEntityResource, PlanEntityAction,
    PlanEntityArgs, PlanEntityResource, VcsEntityAction, VcsEntityArgs, VcsEntityResource,
};
use crate::handlers::{IssueEntityHandler, OrgEntityHandler, PlanEntityHandler, VcsEntityHandler};

/// Unified entity CRUD handler that routes to domain-specific entity handlers.
pub struct EntityHandler {
    vcs: Arc<VcsEntityHandler>,
    plan: Arc<PlanEntityHandler>,
    issue: Arc<IssueEntityHandler>,
    org: Arc<OrgEntityHandler>,
}

impl EntityHandler {
    /// Create a new unified entity handler.
    pub fn new(
        vcs: Arc<VcsEntityHandler>,
        plan: Arc<PlanEntityHandler>,
        issue: Arc<IssueEntityHandler>,
        org: Arc<OrgEntityHandler>,
    ) -> Self {
        Self {
            vcs,
            plan,
            issue,
            org,
        }
    }

    /// Route an `entity` tool call to the matching legacy entity handler.
    // TODO(KISS005): Function handle is too long (78 lines, max: 50).
    // Consider splitting into domain-specific sub-handlers (VCS, Plan, Issue, Org).
    pub async fn handle(
        &self,
        Parameters(args): Parameters<EntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        match args.resource {
            EntityResource::Repository
            | EntityResource::Branch
            | EntityResource::Worktree
            | EntityResource::Assignment => {
                let action = map_vcs_action(args.action).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity action to vcs action: {e}"),
                        None,
                    )
                })?;
                let resource = map_vcs_resource(args.resource).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity resource to vcs resource: {e}"),
                        None,
                    )
                })?;
                self.vcs
                    .handle(Parameters(VcsEntityArgs {
                        action,
                        resource,
                        id: args.id,
                        org_id: args.org_id,
                        project_id: args.project_id,
                        repository_id: args.repository_id,
                        worktree_id: args.worktree_id,
                        data: args.data,
                    }))
                    .await
            }
            EntityResource::Plan | EntityResource::Version | EntityResource::Review => {
                let action = map_standard_action_to_plan(args.action).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity action to plan action: {e}"),
                        None,
                    )
                })?;
                let resource = map_plan_resource(args.resource).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity resource to plan resource: {e}"),
                        None,
                    )
                })?;
                self.plan
                    .handle(Parameters(PlanEntityArgs {
                        action,
                        resource,
                        id: args.id,
                        org_id: args.org_id,
                        project_id: args.project_id,
                        plan_id: args.plan_id,
                        plan_version_id: args.plan_version_id,
                        data: args.data,
                    }))
                    .await
            }
            EntityResource::Issue
            | EntityResource::Comment
            | EntityResource::Label
            | EntityResource::LabelAssignment => {
                let action = map_standard_action_to_issue(args.action).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity action to issue action: {e}"),
                        None,
                    )
                })?;
                let resource = map_issue_resource(args.resource).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity resource to issue resource: {e}"),
                        None,
                    )
                })?;
                self.issue
                    .handle(Parameters(IssueEntityArgs {
                        action,
                        resource,
                        id: args.id,
                        org_id: args.org_id,
                        project_id: args.project_id,
                        issue_id: args.issue_id,
                        label_id: args.label_id,
                        data: args.data,
                    }))
                    .await
            }
            EntityResource::Org
            | EntityResource::User
            | EntityResource::Team
            | EntityResource::TeamMember
            | EntityResource::ApiKey => {
                let action = map_standard_action_to_org(args.action).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity action to org action: {e}"),
                        None,
                    )
                })?;
                let resource = map_org_resource(args.resource).map_err(|e| {
                    McpError::internal_error(
                        format!("failed to map entity resource to org resource: {e}"),
                        None,
                    )
                })?;
                self.org
                    .handle(Parameters(OrgEntityArgs {
                        action,
                        resource,
                        id: args.id,
                        org_id: args.org_id,
                        team_id: args.team_id,
                        user_id: args.user_id,
                        email: args.email,
                        data: args.data,
                    }))
                    .await
            }
        }
    }
}

fn unsupported(msg: &'static str) -> McpError {
    McpError::invalid_params(msg, None)
}

fn map_vcs_action(action: EntityAction) -> Result<VcsEntityAction, McpError> {
    match action {
        EntityAction::Create => Ok(VcsEntityAction::Create),
        EntityAction::Get => Ok(VcsEntityAction::Get),
        EntityAction::Update => Ok(VcsEntityAction::Update),
        EntityAction::List => Ok(VcsEntityAction::List),
        EntityAction::Delete => Ok(VcsEntityAction::Delete),
        EntityAction::Release => Ok(VcsEntityAction::Release),
    }
}

fn map_standard_action_to_plan(action: EntityAction) -> Result<PlanEntityAction, McpError> {
    match action {
        EntityAction::Create => Ok(PlanEntityAction::Create),
        EntityAction::Get => Ok(PlanEntityAction::Get),
        EntityAction::Update => Ok(PlanEntityAction::Update),
        EntityAction::List => Ok(PlanEntityAction::List),
        EntityAction::Delete => Ok(PlanEntityAction::Delete),
        EntityAction::Release => Err(unsupported("release action is only valid for assignment")),
    }
}

fn map_standard_action_to_issue(action: EntityAction) -> Result<IssueEntityAction, McpError> {
    match action {
        EntityAction::Create => Ok(IssueEntityAction::Create),
        EntityAction::Get => Ok(IssueEntityAction::Get),
        EntityAction::Update => Ok(IssueEntityAction::Update),
        EntityAction::List => Ok(IssueEntityAction::List),
        EntityAction::Delete => Ok(IssueEntityAction::Delete),
        EntityAction::Release => Err(unsupported("release action is only valid for assignment")),
    }
}

fn map_standard_action_to_org(action: EntityAction) -> Result<OrgEntityAction, McpError> {
    match action {
        EntityAction::Create => Ok(OrgEntityAction::Create),
        EntityAction::Get => Ok(OrgEntityAction::Get),
        EntityAction::Update => Ok(OrgEntityAction::Update),
        EntityAction::List => Ok(OrgEntityAction::List),
        EntityAction::Delete => Ok(OrgEntityAction::Delete),
        EntityAction::Release => Err(unsupported("release action is only valid for assignment")),
    }
}

fn map_vcs_resource(resource: EntityResource) -> Result<VcsEntityResource, McpError> {
    match resource {
        EntityResource::Repository => Ok(VcsEntityResource::Repository),
        EntityResource::Branch => Ok(VcsEntityResource::Branch),
        EntityResource::Worktree => Ok(VcsEntityResource::Worktree),
        EntityResource::Assignment => Ok(VcsEntityResource::Assignment),
        _ => Err(unsupported(
            "resource is not valid for vcs entity operation",
        )),
    }
}

fn map_plan_resource(resource: EntityResource) -> Result<PlanEntityResource, McpError> {
    match resource {
        EntityResource::Plan => Ok(PlanEntityResource::Plan),
        EntityResource::Version => Ok(PlanEntityResource::Version),
        EntityResource::Review => Ok(PlanEntityResource::Review),
        _ => Err(unsupported(
            "resource is not valid for plan entity operation",
        )),
    }
}

fn map_issue_resource(resource: EntityResource) -> Result<IssueEntityResource, McpError> {
    match resource {
        EntityResource::Issue => Ok(IssueEntityResource::Issue),
        EntityResource::Comment => Ok(IssueEntityResource::Comment),
        EntityResource::Label => Ok(IssueEntityResource::Label),
        EntityResource::LabelAssignment => Ok(IssueEntityResource::LabelAssignment),
        _ => Err(unsupported(
            "resource is not valid for issue entity operation",
        )),
    }
}

fn map_org_resource(resource: EntityResource) -> Result<OrgEntityResource, McpError> {
    match resource {
        EntityResource::Org => Ok(OrgEntityResource::Org),
        EntityResource::User => Ok(OrgEntityResource::User),
        EntityResource::Team => Ok(OrgEntityResource::Team),
        EntityResource::TeamMember => Ok(OrgEntityResource::TeamMember),
        EntityResource::ApiKey => Ok(OrgEntityResource::ApiKey),
        _ => Err(unsupported(
            "resource is not valid for org entity operation",
        )),
    }
}
