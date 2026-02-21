//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin browse endpoints (Axum)
//!
//! Provides endpoints for browsing projects, repositories, plans, issues, and organizations.

use std::sync::Arc;

use axum::Json as AxumJson;
use axum::extract::State as AxumState;
use mcb_domain::error;

use crate::admin::auth::AxumAdminAuth;
use crate::admin::browse_models::{
    IssuesBrowseResponse, OrganizationsBrowseResponse, PlansBrowseResponse, ProjectsBrowseResponse,
    RepositoriesBrowseResponse,
};
use crate::admin::browse_runtime::execute_tool_json;
use crate::admin::error::{AdminError, AdminResult};
use crate::admin::handlers::AdminState;

/// Query parameters for project-scoped browse endpoints.
#[derive(Debug, serde::Deserialize)]
pub struct ProjectIdQuery {
    /// Optional project ID to scope the browse results.
    pub project_id: Option<String>,
}

async fn fetch_browse_items_axum<T: serde::de::DeserializeOwned>(
    state: &AdminState,
    tool_name: &str,
    args: serde_json::Value,
    unavailable_message: &str,
) -> Result<Vec<T>, AdminError> {
    execute_tool_json::<Vec<T>>(state, tool_name, args)
        .await
        .map_err(|e| {
            error!(
                "Browse",
                "failed to list browse items via unified execution", &e
            );
            AdminError::unavailable(unavailable_message)
        })
}

fn build_project_list_args(
    resource: &'static str,
    project_id: Option<String>,
) -> serde_json::Value {
    serde_json::json!({
        "action": "list",
        "resource": resource,
        "project_id": project_id.unwrap_or_default(),
    })
}

async fn fetch_project_scoped_entities_axum<T: serde::de::DeserializeOwned>(
    state: &AdminState,
    resource: &'static str,
    project_id: Option<String>,
    unavailable_message: &'static str,
) -> Result<Vec<T>, AdminError> {
    fetch_browse_items_axum::<T>(
        state,
        "entity",
        build_project_list_args(resource, project_id),
        unavailable_message,
    )
    .await
}

fn build_browse_response_axum<T, R>(
    items: Vec<T>,
    map: impl FnOnce(Vec<T>, usize) -> R,
) -> AxumJson<R> {
    let total = items.len();
    AxumJson(map(items, total))
}

/// List workflow projects for browse entity graph.
///
/// # Errors
/// Returns `503 Service Unavailable` when the backend service is unavailable.
pub async fn list_browse_projects(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminResult<ProjectsBrowseResponse> {
    tracing::info!("list_browse_projects called");
    let args = build_project_list_args("project", None);

    let projects = fetch_browse_items_axum::<mcb_domain::entities::project::Project>(
        &state,
        "project",
        args,
        "Project workflow service not available",
    )
    .await?;

    Ok(build_browse_response_axum(projects, |projects, total| {
        ProjectsBrowseResponse { projects, total }
    }))
}

define_project_scoped_browse_endpoint!(
    list_browse_repositories,
    mcb_domain::entities::repository::Repository,
    RepositoriesBrowseResponse,
    repositories,
    "repository",
    "VCS entity service not available",
    "list_browse_repositories called",
    "List repositories for browse entity graph."
);

define_project_scoped_browse_endpoint!(
    list_browse_plans,
    mcb_domain::entities::plan::Plan,
    PlansBrowseResponse,
    plans,
    "plan",
    "Plan entity service not available",
    "list_browse_plans called",
    "List plans for browse entity graph."
);

define_project_scoped_browse_endpoint!(
    list_browse_issues,
    mcb_domain::entities::project::ProjectIssue,
    IssuesBrowseResponse,
    issues,
    "issue",
    "Issue entity service not available",
    "list_browse_issues called",
    "List issues for browse entity graph."
);

/// List organizations for browse entity graph.
///
/// # Errors
/// Returns `503 Service Unavailable` when the backend service is unavailable.
pub async fn list_browse_organizations(
    _auth: AxumAdminAuth,
    AxumState(state): AxumState<Arc<AdminState>>,
) -> AdminResult<OrganizationsBrowseResponse> {
    let args = build_project_list_args("org", None);

    let organizations =
        fetch_browse_items_axum::<mcb_domain::entities::organization::Organization>(
            &state,
            "entity",
            args,
            "Org entity service not available",
        )
        .await?;

    Ok(build_browse_response_axum(
        organizations,
        |organizations, total| OrganizationsBrowseResponse {
            organizations,
            total,
        },
    ))
}
