use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::{ImpactFile, ImpactResponse, ImpactSummary, repo_path};
use crate::args::VcsArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;

/// Analyzes the impact of changes between branches.
#[tracing::instrument(skip_all)]
pub async fn analyze_impact(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let path = match repo_path(args) {
        Ok(p) => p,
        Err(error_result) => return Ok(error_result),
    };
    let base_ref = args
        .base_branch
        .clone()
        .unwrap_or_else(|| "main".to_string());
    let head_ref = args
        .target_branch
        .clone()
        .unwrap_or_else(|| "HEAD".to_string());
    let repo = match vcs_provider.open_repository(Path::new(&path)).await {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let diff = match vcs_provider.diff_refs(&repo, &base_ref, &head_ref).await {
        Ok(diff) => diff,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let mut added = 0;
    let mut modified = 0;
    let mut deleted = 0;
    let mut impacted_files = Vec::new();
    for file in diff.files.iter() {
        let status = file.status.to_string();
        match status.as_str() {
            "added" => added += 1,
            "deleted" => deleted += 1,
            _ => modified += 1,
        }
        impacted_files.push(ImpactFile {
            path: file.path.to_str().unwrap_or_default().to_string(),
            status: status.clone(),
            impact: file.additions + file.deletions,
        });
    }
    let total_changes = diff.total_additions + diff.total_deletions;
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
