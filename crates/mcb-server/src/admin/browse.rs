//! Admin browse endpoints
//!
//! Provides endpoints for browsing projects, repositories, plans, issues, and organizations.

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

/// List workflow projects for browse entity graph
#[get("/projects")]
pub async fn list_browse_projects(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<ProjectsBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    tracing::info!("list_browse_projects called");
    let args = serde_json::json!({
        "action": "list",
        "resource": "project",
        "project_id": ""
    });

    let projects: Vec<mcb_domain::entities::project::Project> =
        match execute_tool_json(state.inner(), "project", args).await {
            Ok(projects) => projects,
            Err(e) => {
                tracing::error!(error = %e, "failed to list projects via unified execution");
                return Err((
                    Status::ServiceUnavailable,
                    Json(CacheErrorResponse {
                        error: "Project workflow service not available".to_string(),
                    }),
                ));
            }
        };

    let total = projects.len();
    Ok(Json(ProjectsBrowseResponse { projects, total }))
}

/// List repositories for browse entity graph.
#[get("/repositories?<project_id>")]
pub async fn list_browse_repositories(
    _auth: AdminAuth,
    state: &State<AdminState>,
    project_id: Option<String>,
) -> Result<Json<RepositoriesBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    tracing::info!("list_browse_repositories called");
    let args = serde_json::json!({
        "action": "list",
        "resource": "repository",
        "project_id": project_id.unwrap_or_default()
    });

    let repositories: Vec<mcb_domain::entities::repository::Repository> =
        match execute_tool_json(state.inner(), "entity", args).await {
            Ok(repositories) => repositories,
            Err(e) => {
                tracing::error!(error = %e, "failed to list repositories via unified execution");
                return Err((
                    Status::ServiceUnavailable,
                    Json(CacheErrorResponse {
                        error: "VCS entity service not available".to_string(),
                    }),
                ));
            }
        };

    let total = repositories.len();
    Ok(Json(RepositoriesBrowseResponse {
        repositories,
        total,
    }))
}

/// List plans for browse entity graph.
#[get("/plans?<project_id>")]
pub async fn list_browse_plans(
    _auth: AdminAuth,
    state: &State<AdminState>,
    project_id: Option<String>,
) -> Result<Json<PlansBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    tracing::info!("list_browse_plans called");
    let args = serde_json::json!({
        "action": "list",
        "resource": "plan",
        "project_id": project_id.unwrap_or_default()
    });

    let plans: Vec<mcb_domain::entities::plan::Plan> =
        match execute_tool_json(state.inner(), "entity", args).await {
            Ok(plans) => plans,
            Err(e) => {
                tracing::error!(error = %e, "failed to list plans via unified execution");
                return Err((
                    Status::ServiceUnavailable,
                    Json(CacheErrorResponse {
                        error: "Plan entity service not available".to_string(),
                    }),
                ));
            }
        };

    let total = plans.len();
    Ok(Json(PlansBrowseResponse { plans, total }))
}

/// List issues for browse entity graph.
#[get("/issues?<project_id>")]
pub async fn list_browse_issues(
    _auth: AdminAuth,
    state: &State<AdminState>,
    project_id: Option<String>,
) -> Result<Json<IssuesBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    tracing::info!("list_browse_issues called");
    let args = serde_json::json!({
        "action": "list",
        "resource": "issue",
        "project_id": project_id.unwrap_or_default()
    });

    let issues: Vec<mcb_domain::entities::project::ProjectIssue> =
        match execute_tool_json(state.inner(), "entity", args).await {
            Ok(issues) => issues,
            Err(e) => {
                tracing::error!(error = %e, "failed to list issues via unified execution");
                return Err((
                    Status::ServiceUnavailable,
                    Json(CacheErrorResponse {
                        error: "Issue entity service not available".to_string(),
                    }),
                ));
            }
        };

    let total = issues.len();
    Ok(Json(IssuesBrowseResponse { issues, total }))
}

/// List organizations for browse entity graph.
#[get("/organizations")]
pub async fn list_browse_organizations(
    _auth: AdminAuth,
    state: &State<AdminState>,
) -> Result<Json<OrganizationsBrowseResponse>, (Status, Json<CacheErrorResponse>)> {
    let args = serde_json::json!({
        "action": "list",
        "resource": "org"
    });

    let organizations: Vec<mcb_domain::entities::organization::Organization> =
        match execute_tool_json(state.inner(), "entity", args).await {
            Ok(organizations) => organizations,
            Err(e) => {
                tracing::error!(error = %e, "failed to list organizations via unified execution");
                return Err((
                    Status::ServiceUnavailable,
                    Json(CacheErrorResponse {
                        error: "Org entity service not available".to_string(),
                    }),
                ));
            }
        };

    let total = organizations.len();
    Ok(Json(OrganizationsBrowseResponse {
        organizations,
        total,
    }))
}
