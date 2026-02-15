use serde::Serialize;

/// Projects list response for browse entity navigation.
#[derive(Serialize)]
pub struct ProjectsBrowseResponse {
    /// List of projects.
    pub projects: Vec<mcb_domain::entities::project::Project>,
    /// Total number of projects.
    pub total: usize,
}

/// Response payload for the repositories browse endpoint.
#[derive(Serialize)]
pub struct RepositoriesBrowseResponse {
    /// List of repositories.
    pub repositories: Vec<mcb_domain::entities::repository::Repository>,
    /// Total number of repositories.
    pub total: usize,
}

/// Response payload for the plans browse endpoint.
#[derive(Serialize)]
pub struct PlansBrowseResponse {
    /// List of plans.
    pub plans: Vec<mcb_domain::entities::plan::Plan>,
    /// Total number of plans.
    pub total: usize,
}

/// Response payload for the issues browse endpoint.
#[derive(Serialize)]
pub struct IssuesBrowseResponse {
    /// List of issues.
    pub issues: Vec<mcb_domain::entities::project::ProjectIssue>,
    /// Total number of issues.
    pub total: usize,
}

/// Response payload for the organizations browse endpoint.
#[derive(Serialize)]
pub struct OrganizationsBrowseResponse {
    /// List of organizations.
    pub organizations: Vec<mcb_domain::entities::organization::Organization>,
    /// Total number of organizations.
    pub total: usize,
}
