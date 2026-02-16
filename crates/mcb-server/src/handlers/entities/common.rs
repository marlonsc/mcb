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
    ///
    /// # Errors
    /// Returns an error when action/resource mapping is invalid for the target domain handler.
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

define_action_mapper!(map_vcs_action, VcsEntityAction, allow_all);
define_action_mapper!(
    map_standard_action_to_plan,
    PlanEntityAction,
    reject_release
);
define_action_mapper!(
    map_standard_action_to_issue,
    IssueEntityAction,
    reject_release
);
define_action_mapper!(map_standard_action_to_org, OrgEntityAction, reject_release);

define_resource_mapper!(map_vcs_resource, VcsEntityResource,
    "resource is not valid for vcs entity operation", {
    Repository => Repository, Branch => Branch,
    Worktree => Worktree, Assignment => Assignment,
});

define_resource_mapper!(map_plan_resource, PlanEntityResource,
    "resource is not valid for plan entity operation", {
    Plan => Plan, Version => Version, Review => Review,
});

define_resource_mapper!(map_issue_resource, IssueEntityResource,
    "resource is not valid for issue entity operation", {
    Issue => Issue, Comment => Comment,
    Label => Label, LabelAssignment => LabelAssignment,
});

define_resource_mapper!(map_org_resource, OrgEntityResource,
    "resource is not valid for org entity operation", {
    Org => Org, User => User, Team => Team,
    TeamMember => TeamMember, ApiKey => ApiKey,
});
