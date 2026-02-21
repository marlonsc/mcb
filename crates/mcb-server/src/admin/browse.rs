//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Admin browse endpoints
//!
//! Provides endpoints for browsing projects, repositories, plans, issues, and organizations.

use std::sync::Arc;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};

use crate::admin::auth::AdminAuth;
use crate::admin::browse_models::{
    IssuesBrowseResponse, OrganizationsBrowseResponse, PlansBrowseResponse, ProjectsBrowseResponse,
    RepositoriesBrowseResponse,
};
use crate::admin::browse_runtime::execute_tool_json;
use crate::admin::handlers::{AdminState, CacheErrorResponse};

/// Query parameters for project-scoped browse endpoints.
#[derive(Debug, serde::Deserialize)]
pub struct ProjectIdQuery {
    /// Optional project ID to scope the browse results.
    pub project_id: Option<String>,
}

async fn fetch_browse_items<T: serde::de::DeserializeOwned>(
    state: &AdminState,
    tool_name: &str,
    args: serde_json::Value,
    unavailable_message: &str,
) -> Result<Vec<T>, (Status, Json<CacheErrorResponse>)> {
    execute_tool_json::<Vec<T>>(state, tool_name, args)
        .await
        .map_err(|_error| {
            tracing::error!("failed to list browse items via unified execution");
            (
                Status::ServiceUnavailable,
                Json(CacheErrorResponse {
                    error: unavailable_message.to_owned(),
                }),
            )
        })
}

async fn fetch_browse_items_axum<T: serde::de::DeserializeOwned>(
    state: &AdminState,
    tool_name: &str,
    args: serde_json::Value,
    unavailable_message: &str,
) -> Result<Vec<T>, (axum::http::StatusCode, axum::Json<CacheErrorResponse>)> {
    execute_tool_json::<Vec<T>>(state, tool_name, args)
        .await
        .map_err(|_error| {
            tracing::error!("failed to list browse items via unified execution");
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(CacheErrorResponse {
                    error: unavailable_message.to_owned(),
                }),
            )
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
) -> Result<Vec<T>, (axum::http::StatusCode, axum::Json<CacheErrorResponse>)> {
    fetch_browse_items_axum::<T>(
        state,
        "entity",
        build_project_list_args(resource, project_id),
        unavailable_message,
    )
    .await
}

fn build_browse_response<T, R>(items: Vec<T>, map: impl FnOnce(Vec<T>, usize) -> R) -> Json<R> {
    let total = items.len();
    Json(map(items, total))
}

fn build_browse_response_axum<T, R>(
    items: Vec<T>,
    map: impl FnOnce(Vec<T>, usize) -> R,
) -> axum::Json<R> {
    let total = items.len();
    axum::Json(map(items, total))
}

/// List workflow projects for browse entity graph
///
/// # Errors
/// Returns `503 Service Unavailable` when the backend service is unavailable.
#[get("/projects")]
pub async fn list_browse_projects(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ProjectsBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    tracing::info!("list_browse_projects called");
    let args = build_project_list_args("project", None);

    let projects = fetch_browse_items::<mcb_domain::entities::project::Project>(
        state.inner(),
        "project",
        args,
        "Project workflow service not available",
    )
    .await?;

    Ok(build_browse_response(projects, |projects, total| {
        ProjectsBrowseResponse { projects, total }
    }))
}

/// Axum handler: list workflow projects for browse entity graph.
///
/// # Errors
/// Returns `503 Service Unavailable` when the backend service is unavailable.
pub async fn list_browse_projects_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
) -> Result<
    axum::Json<ProjectsBrowseResponse>,
    (axum::http::StatusCode, axum::Json<CacheErrorResponse>),
> {
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
    "/repositories?<project_id>",
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
    "/plans?<project_id>",
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
    "/issues?<project_id>",
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
#[get("/organizations")]
pub async fn list_browse_organizations(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<OrganizationsBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    let args = build_project_list_args("org", None);

    let organizations = fetch_browse_items::<mcb_domain::entities::organization::Organization>(
        state.inner(),
        "entity",
        args,
        "Org entity service not available",
    )
    .await?;

    Ok(build_browse_response(
        organizations,
        |organizations, total| OrganizationsBrowseResponse {
            organizations,
            total,
        },
    ))
}

/// Axum handler: list organizations for browse entity graph.
///
/// # Errors
/// Returns `503 Service Unavailable` when the backend service is unavailable.
pub async fn list_browse_organizations_axum(
    _auth: crate::admin::auth::AxumAdminAuth,
    axum::extract::State(state): axum::extract::State<Arc<AdminState>>,
) -> Result<
    axum::Json<OrganizationsBrowseResponse>,
    (axum::http::StatusCode, axum::Json<CacheErrorResponse>),
> {
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
