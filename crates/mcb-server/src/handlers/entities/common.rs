//! Unified entity CRUD handler implementation.

use std::future::Future;
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
    #[must_use]
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
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<EntityArgs>,
    ) -> Result<CallToolResult, McpError> {
        match args.resource {
            EntityResource::Repository
            | EntityResource::Branch
            | EntityResource::Worktree
            | EntityResource::Assignment => self.route_vcs(args).await,

            EntityResource::Plan | EntityResource::Version | EntityResource::Review => {
                self.route_plan(args).await
            }

            EntityResource::Issue
            | EntityResource::Comment
            | EntityResource::Label
            | EntityResource::LabelAssignment => self.route_issue(args).await,

            EntityResource::Org
            | EntityResource::User
            | EntityResource::Team
            | EntityResource::TeamMember
            | EntityResource::ApiKey => self.route_org(args).await,
        }
    }

    define_route_method!(
        route_vcs,
        vcs,
        VcsEntityArgs,
        map_vcs_action,
        map_vcs_resource,
        |args, action, resource| VcsEntityArgs {
            action,
            resource,
            id: args.id,
            org_id: args.org_id,
            project_id: args.project_id,
            repository_id: args.repository_id,
            worktree_id: args.worktree_id,
            data: args.data,
        }
    );

    define_route_method!(
        route_plan,
        plan,
        PlanEntityArgs,
        map_standard_action_to_plan,
        map_plan_resource,
        |args, action, resource| PlanEntityArgs {
            action,
            resource,
            id: args.id,
            org_id: args.org_id,
            project_id: args.project_id,
            plan_id: args.plan_id,
            plan_version_id: args.plan_version_id,
            data: args.data,
        }
    );

    define_route_method!(
        route_issue,
        issue,
        IssueEntityArgs,
        map_standard_action_to_issue,
        map_issue_resource,
        |args, action, resource| IssueEntityArgs {
            action,
            resource,
            id: args.id,
            org_id: args.org_id,
            project_id: args.project_id,
            issue_id: args.issue_id,
            label_id: args.label_id,
            data: args.data,
        }
    );

    define_route_method!(
        route_org,
        org,
        OrgEntityArgs,
        map_standard_action_to_org,
        map_org_resource,
        |args, action, resource| OrgEntityArgs {
            action,
            resource,
            id: args.id,
            org_id: args.org_id,
            team_id: args.team_id,
            user_id: args.user_id,
            email: args.email,
            data: args.data,
        }
    );

    async fn route_entity<
        RawAction,
        RawResource,
        RoutedArgs,
        MapAction,
        MapResource,
        BuildArgs,
        HandleFn,
        HandleFuture,
    >(
        &self,
        args: EntityArgs,
        map_action: MapAction,
        map_resource: MapResource,
        build_args: BuildArgs,
        handle: HandleFn,
    ) -> Result<CallToolResult, McpError>
    where
        MapAction: FnOnce(EntityAction) -> Result<RawAction, McpError>,
        MapResource: FnOnce(EntityResource) -> Result<RawResource, McpError>,
        BuildArgs: FnOnce(EntityArgs, RawAction, RawResource) -> RoutedArgs,
        HandleFn: FnOnce(RoutedArgs) -> HandleFuture,
        HandleFuture: Future<Output = Result<CallToolResult, McpError>>,
    {
        let action = map_action(args.action)?;
        let resource = map_resource(args.resource)?;
        let routed_args = build_args(args, action, resource);
        handle(routed_args).await
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
