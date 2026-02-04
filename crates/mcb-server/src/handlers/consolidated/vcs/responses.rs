use crate::args::VcsArgs;
use crate::vcs_repository_registry;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct ListRepositoriesResponse {
    pub repositories: Vec<String>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct IndexResult {
    pub repository_id: String,
    pub path: String,
    pub default_branch: String,
    pub branches_found: Vec<String>,
    pub total_files: usize,
    pub commits_indexed: usize,
}

#[derive(Serialize)]
pub struct BranchComparison {
    pub base_branch: String,
    pub head_branch: String,
    pub files_changed: usize,
    pub additions: usize,
    pub deletions: usize,
    pub files: Vec<BranchDiffFile>,
}

#[derive(Serialize)]
pub struct BranchDiffFile {
    pub path: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct BranchSearchResponse {
    pub repository_id: String,
    pub branch: String,
    pub query: String,
    pub count: usize,
    pub results: Vec<BranchSearchMatch>,
}

#[derive(Serialize)]
pub struct BranchSearchMatch {
    pub path: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(Serialize)]
pub struct ImpactSummary {
    pub total_files: usize,
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub total_changes: usize,
}

#[derive(Serialize)]
pub struct ImpactFile {
    pub path: String,
    pub status: String,
    pub impact: usize,
}

#[derive(Serialize)]
pub struct ImpactResponse {
    pub base_ref: String,
    pub head_ref: String,
    pub impact_score: f64,
    pub summary: ImpactSummary,
    pub impacted_files: Vec<ImpactFile>,
}

pub fn repo_path(args: &VcsArgs) -> Result<PathBuf, CallToolResult> {
    if let Some(path) = args.repo_path.as_ref() {
        return Ok(PathBuf::from(path));
    }
    if let Some(repo_id) = args.repo_id.as_ref() {
        match vcs_repository_registry::lookup_repository_path(repo_id) {
            Ok(path) => return Ok(path),
            Err(_) => {
                return Err(CallToolResult::error(vec![Content::text(format!(
                    "Repository not found: {repo_id}"
                ))]));
            }
        }
    }
    Err(CallToolResult::error(vec![Content::text(
        "Missing repo_path or repo_id",
    )]))
}
