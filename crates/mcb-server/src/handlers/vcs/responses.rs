use std::path::PathBuf;

use rmcp::model::CallToolResult;
use serde::Serialize;

use crate::args::VcsArgs;
use crate::handlers::helpers::tool_error;

/// Response structure for listing repositories.
#[derive(Serialize)]
pub struct ListRepositoriesResponse {
    /// List of repository identifiers or names.
    pub repositories: Vec<String>,
    /// Total number of repositories found.
    pub count: usize,
}

/// Result of a repository indexing operation.
#[derive(Serialize)]
pub struct IndexResult {
    /// Identifier of the indexed repository.
    pub repository_id: String,
    /// Filesystem path to the repository.
    pub path: String,
    /// Name of the default branch (e.g., main, master).
    pub default_branch: String,
    /// List of branches discovered in the repository.
    pub branches_found: Vec<String>,
    /// Total number of files indexed.
    pub total_files: usize,
    /// Number of commits processed during indexing.
    pub commits_indexed: usize,
}

/// Result of comparing two branches.
#[derive(Serialize)]
pub struct BranchComparison {
    /// Name of the base branch.
    pub base_branch: String,
    /// Name of the head branch.
    pub head_branch: String,
    /// Number of files with differences.
    pub files_changed: usize,
    /// Total number of lines added.
    pub additions: usize,
    /// Total number of lines deleted.
    pub deletions: usize,
    /// Detailed list of changed files.
    pub files: Vec<BranchDiffFile>,
}

/// Details of a file changed between branches.
#[derive(Serialize)]
pub struct BranchDiffFile {
    /// Path to the changed file.
    pub path: String,
    /// Status of the change (e.g., added, modified, deleted).
    pub status: String,
}

/// Response structure for searching within a branch.
#[derive(Serialize)]
pub struct BranchSearchResponse {
    /// Identifier of the repository searched.
    pub repository_id: String,
    /// Name of the branch searched.
    pub branch: String,
    /// The search query string.
    pub query: String,
    /// Number of matches found.
    pub count: usize,
    /// List of search results.
    pub results: Vec<BranchSearchMatch>,
}

/// A specific match found during a branch search.
#[derive(Serialize)]
pub struct BranchSearchMatch {
    /// Path to the file containing the match.
    pub path: String,
    /// Line number where the match occurs.
    pub line: usize,
    /// Code snippet containing the match.
    pub snippet: String,
}

/// Summary of change impact analysis.
#[derive(Serialize)]
pub struct ImpactSummary {
    /// Total number of files affected.
    pub total_files: usize,
    /// Number of files added.
    pub added: usize,
    /// Number of files modified.
    pub modified: usize,
    /// Number of files deleted.
    pub deleted: usize,
    /// Total magnitude of changes (e.g., lines changed).
    pub total_changes: usize,
}

/// Impact details for a specific file.
#[derive(Serialize)]
pub struct ImpactFile {
    /// Path to the impacted file.
    pub path: String,
    /// Status of the file change.
    pub status: String,
    /// Calculated impact score for this file.
    pub impact: usize,
}

/// Response structure for impact analysis between references.
#[derive(Serialize)]
pub struct ImpactResponse {
    /// Base reference (branch/commit) for comparison.
    pub base_ref: String,
    /// Head reference (branch/commit) for comparison.
    pub head_ref: String,
    /// Calculated overall impact score.
    pub impact_score: f64,
    /// Summary of the impact statistics.
    pub summary: ImpactSummary,
    /// List of files with their impact details.
    pub impacted_files: Vec<ImpactFile>,
}

/// Resolves the repository path from arguments.
///
/// # Arguments
///
/// * `args` - VCS arguments containing either `repo_path` or `repo_id`.
///
/// # Returns
///
/// * `Ok(PathBuf)` - The resolved filesystem path to the repository.
/// * `Err(CallToolResult)` - Error if the repository cannot be found or arguments are missing.
pub fn repo_path(args: &VcsArgs) -> Result<PathBuf, CallToolResult> {
    if let Some(path) = args.repo_path.as_ref() {
        return Ok(PathBuf::from(path));
    }
    if let Some(repo_id) = args.repo_id.as_ref() {
        return Err(tool_error(format!(
            "Repository not found: {repo_id}. Provide repo_path instead."
        )));
    }
    Err(tool_error("Missing repo_path or repo_id"))
}
