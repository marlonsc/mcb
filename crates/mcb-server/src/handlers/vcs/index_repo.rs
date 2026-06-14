//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use mcb_domain::ports::VcsProvider;

use super::responses::{IndexResult, repo_path};
use crate::args::VcsArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;

/// Indexes a repository for search.
#[tracing::instrument(skip_all)]
pub async fn index_repository(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let path = match repo_path(args) {
        Ok(p) => p,
        Err(error_result) => return Ok(error_result),
    };
    let repo = match vcs_provider.open_repository(Path::new(&path)).await {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };

    // Load config from repository path
    let config = load_repo_context_config(Path::new(&path));

    // Determine depth: args.depth > config.git.depth > default 1000
    let depth = args.depth.unwrap_or(config.git.depth);

    let branches = args
        .branches
        .clone()
        .unwrap_or_else(|| vec![repo.default_branch().to_owned()]);
    let total_files =
        match count_filtered_files(vcs_provider, &repo, &branches, &config.git.ignore_patterns)
            .await
        {
            Ok(count) => count,
            Err(e) => return Ok(to_contextual_tool_error(e)),
        };
    let commits_indexed = if args.include_commits.unwrap_or(false) {
        count_commits(vcs_provider, &repo, &branches, depth).await
    } else {
        0
    };
    let result = IndexResult {
        repository_id: repo.id().to_string(),
        path: repo.path().to_string_lossy().into_owned(),
        default_branch: repo.default_branch().to_owned(),
        branches_found: branches.clone(),
        total_files,
        commits_indexed,
    };

    ResponseFormatter::json_success(&result)
}

/// Count files across `branches` after applying ignore patterns.
async fn count_filtered_files(
    vcs_provider: &Arc<dyn VcsProvider>,
    repo: &mcb_domain::entities::vcs::VcsRepository,
    branches: &[String],
    ignore_patterns: &[String],
) -> mcb_domain::Result<usize> {
    let mut total_files = 0;
    for branch in branches {
        let files = vcs_provider.list_files(repo, branch).await?;
        total_files += filter_files_by_patterns(&files, ignore_patterns).len();
    }
    Ok(total_files)
}

/// Count commits across `branches`; per-branch failures are logged and skipped.
async fn count_commits(
    vcs_provider: &Arc<dyn VcsProvider>,
    repo: &mcb_domain::entities::vcs::VcsRepository,
    branches: &[String],
    depth: usize,
) -> usize {
    let mut count = 0;
    for branch in branches {
        match vcs_provider.commit_history(repo, branch, Some(depth)).await {
            Ok(commits) => count += commits.len(),
            Err(e) => {
                mcb_domain::warn!(
                    "vcs",
                    "Failed to index commits",
                    &format!("branch={branch}: {e}")
                );
            }
        }
    }
    count
}

/// Filter files by ignore patterns (gitignore-style)
fn filter_files_by_patterns(files: &[PathBuf], patterns: &[String]) -> Vec<PathBuf> {
    if patterns.is_empty() {
        return files.to_vec();
    }

    files
        .iter()
        .filter(|file| !should_ignore_file(file, patterns))
        .cloned()
        .collect()
}

/// Check if a file should be ignored based on patterns
fn should_ignore_file(file: &Path, patterns: &[String]) -> bool {
    let Some(file_str) = file.to_str() else {
        return false;
    };
    patterns
        .iter()
        .any(|pattern| pattern_matches(file_str, pattern))
}

/// Match a single gitignore-style pattern against a file path string.
fn pattern_matches(file_str: &str, pattern: &str) -> bool {
    // Directory patterns end with `/`.
    if let Some(dir_pattern) = pattern.strip_suffix('/') {
        return file_str.contains(dir_pattern);
    }
    // Wildcard patterns start with `*` (e.g. `*.ext`).
    if let Some(ext) = pattern.strip_prefix('*') {
        return file_str.ends_with(ext);
    }
    // Exact matches and path patterns.
    file_str.contains(pattern) || file_str.ends_with(pattern)
}

// ---------------------------------------------------------------------------
// Repository-local .mcp-context.toml config (inlined to avoid cross-crate dep)
// ---------------------------------------------------------------------------

/// Git section of .mcp-context.toml.
#[derive(serde::Deserialize)]
struct GitContextConfig {
    #[serde(default = "default_git_depth")]
    depth: usize,
    #[serde(default)]
    ignore_patterns: Vec<String>,
}

fn default_git_depth() -> usize {
    50
}

impl Default for GitContextConfig {
    fn default() -> Self {
        Self {
            depth: default_git_depth(),
            ignore_patterns: Vec::new(),
        }
    }
}

/// Root .mcp-context.toml config.
#[derive(Default, serde::Deserialize)]
struct RepoContextConfig {
    #[serde(default)]
    git: GitContextConfig,
}

/// Load `.mcp-context.toml` from the given directory, returning defaults on any error.
fn load_repo_context_config(path: &Path) -> RepoContextConfig {
    let config_path = path.join(".mcp-context.toml");
    std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}
