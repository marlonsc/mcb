use crate::args::{VcsAction, VcsArgs};
use crate::collection_mapping;
use crate::formatter::ResponseFormatter;
use crate::vcs_repository_registry;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use validator::Validate;

#[derive(Clone)]
pub struct VcsHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

#[derive(Serialize)]
struct ListRepositoriesResponse {
    repositories: Vec<String>,
    count: usize,
}

#[derive(Serialize)]
struct IndexResult {
    repository_id: String,
    path: String,
    default_branch: String,
    branches_found: Vec<String>,
    total_files: usize,
    commits_indexed: usize,
}

#[derive(Serialize)]
struct BranchComparison {
    base_branch: String,
    head_branch: String,
    files_changed: usize,
    additions: usize,
    deletions: usize,
    files: Vec<BranchDiffFile>,
}

#[derive(Serialize)]
struct BranchDiffFile {
    path: String,
    status: String,
}

#[derive(Serialize)]
struct BranchSearchResponse {
    repository_id: String,
    branch: String,
    query: String,
    count: usize,
    results: Vec<BranchSearchMatch>,
}

#[derive(Serialize)]
struct BranchSearchMatch {
    path: String,
    line: usize,
    snippet: String,
}

#[derive(Serialize)]
struct ImpactSummary {
    total_files: usize,
    added: usize,
    modified: usize,
    deleted: usize,
    total_changes: usize,
}

#[derive(Serialize)]
struct ImpactFile {
    path: String,
    status: String,
    impact: usize,
}

#[derive(Serialize)]
struct ImpactResponse {
    base_ref: String,
    head_ref: String,
    impact_score: f64,
    summary: ImpactSummary,
    impacted_files: Vec<ImpactFile>,
}

impl VcsHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    fn repo_path(args: &VcsArgs) -> Result<PathBuf, CallToolResult> {
        if let Some(path) = args.repo_path.as_ref() {
            return Ok(PathBuf::from(path));
        }
        if let Some(repo_id) = args.repo_id.as_ref() {
            if let Some(path) = vcs_repository_registry::lookup_repository_path(repo_id) {
                return Ok(PathBuf::from(path));
            }
            return Err(CallToolResult::error(vec![Content::text(format!(
                "Repository not found: {repo_id}"
            ))]));
        }
        Err(CallToolResult::error(vec![Content::text(
            "Missing repo_path or repo_id",
        )]))
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<VcsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            VcsAction::ListRepositories => {
                let repositories = collection_mapping::list_collections().map_err(|e| {
                    McpError::internal_error(format!("Failed to list collections: {e}"), None)
                })?;
                let result = ListRepositoriesResponse {
                    count: repositories.len(),
                    repositories,
                };
                ResponseFormatter::json_success(&result)
            }
            VcsAction::IndexRepository => {
                let path = Self::repo_path(&args)?;
                let repo = match self.vcs_provider.open_repository(Path::new(&path)).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to open repository: {e}"
                        ))]));
                    }
                };
                let branches = args
                    .branches
                    .clone()
                    .unwrap_or_else(|| vec![repo.default_branch.clone()]);
                let mut total_files = 0;
                for branch in &branches {
                    match self.vcs_provider.list_files(&repo, branch).await {
                        Ok(files) => total_files += files.len(),
                        Err(e) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Failed to list files in branch {branch}: {e}"
                            ))]));
                        }
                    }
                }
                let commits_indexed = if args.include_commits.unwrap_or(false) {
                    let mut count = 0;
                    for branch in &branches {
                        if let Ok(commits) = self
                            .vcs_provider
                            .commit_history(&repo, branch, Some(1000))
                            .await
                        {
                            count += commits.len();
                        }
                    }
                    count
                } else {
                    0
                };
                let result = IndexResult {
                    repository_id: repo.id.clone(),
                    path: repo.path.clone(),
                    default_branch: repo.default_branch.clone(),
                    branches_found: branches.clone(),
                    total_files,
                    commits_indexed,
                };
                vcs_repository_registry::record_repository(&repo.id, &repo.path);
                ResponseFormatter::json_success(&result)
            }
            VcsAction::CompareBranches => {
                let path = Self::repo_path(&args)?;
                let base = args
                    .base_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string());
                let head = args
                    .target_branch
                    .clone()
                    .unwrap_or_else(|| "HEAD".to_string());
                let repo = match self.vcs_provider.open_repository(Path::new(&path)).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to open repository: {e}"
                        ))]));
                    }
                };
                let diff = match self.vcs_provider.diff_refs(&repo, &base, &head).await {
                    Ok(diff) => diff,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to diff branches: {e}"
                        ))]));
                    }
                };
                let files = diff
                    .files
                    .iter()
                    .map(|file| BranchDiffFile {
                        path: file.path.clone(),
                        status: format!("{:?}", file.status).to_lowercase(),
                    })
                    .collect();
                let result = BranchComparison {
                    base_branch: base,
                    head_branch: head,
                    files_changed: diff.files.len(),
                    additions: diff.additions,
                    deletions: diff.deletions,
                    files,
                };
                ResponseFormatter::json_success(&result)
            }
            VcsAction::SearchBranch => {
                let query = match args.query.as_ref() {
                    Some(value) if !value.trim().is_empty() => value.trim(),
                    _ => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing query for branch search",
                        )]));
                    }
                };
                let branch = args
                    .target_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string());
                let path = Self::repo_path(&args)?;
                let repo = match self.vcs_provider.open_repository(Path::new(&path)).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to open repository: {e}"
                        ))]));
                    }
                };
                let files = match self.vcs_provider.list_files(&repo, &branch).await {
                    Ok(files) => files,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to list files: {e}"
                        ))]));
                    }
                };
                let limit = args.limit.unwrap_or(20) as usize;
                let mut matches = Vec::new();
                for path in files {
                    if matches.len() >= limit {
                        break;
                    }
                    if let Ok(content) = self.vcs_provider.read_file(&repo, &branch, &path).await {
                        for (index, line) in content.lines().enumerate() {
                            if line.to_lowercase().contains(&query.to_lowercase()) {
                                matches.push(BranchSearchMatch {
                                    path: path.clone(),
                                    line: index + 1,
                                    snippet: line.trim().to_string(),
                                });
                                if matches.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
                let result = BranchSearchResponse {
                    repository_id: repo.id.clone(),
                    branch,
                    query: query.to_string(),
                    count: matches.len(),
                    results: matches,
                };
                ResponseFormatter::json_success(&result)
            }
            VcsAction::AnalyzeImpact => {
                let path = Self::repo_path(&args)?;
                let base_ref = args
                    .base_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string());
                let head_ref = args
                    .target_branch
                    .clone()
                    .unwrap_or_else(|| "HEAD".to_string());
                let repo = match self.vcs_provider.open_repository(Path::new(&path)).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to open repository: {e}"
                        ))]));
                    }
                };
                let diff = match self
                    .vcs_provider
                    .diff_refs(&repo, &base_ref, &head_ref)
                    .await
                {
                    Ok(diff) => diff,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to diff branches: {e}"
                        ))]));
                    }
                };
                let mut added = 0;
                let mut modified = 0;
                let mut deleted = 0;
                let mut impacted_files = Vec::new();
                for file in diff.files.iter() {
                    let status = format!("{:?}", file.status).to_lowercase();
                    match status.as_str() {
                        "added" => added += 1,
                        "deleted" => deleted += 1,
                        _ => modified += 1,
                    }
                    impacted_files.push(ImpactFile {
                        path: file.path.clone(),
                        status: status.clone(),
                        impact: file.additions + file.deletions,
                    });
                }
                let total_changes = diff.additions + diff.deletions;
                let impact_score = ((diff.files.len() as f64).ln_1p() * 10.0
                    + (total_changes as f64).ln_1p() * 5.0)
                    .min(100.0);
                let result = ImpactResponse {
                    base_ref,
                    head_ref,
                    impact_score,
                    summary: ImpactSummary {
                        total_files: diff.files.len(),
                        added,
                        modified,
                        deleted,
                        total_changes,
                    },
                    impacted_files,
                };
                ResponseFormatter::json_success(&result)
            }
        }
    }
}
