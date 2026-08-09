//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::{ImpactFile, ImpactResponse, ImpactSummary, repo_path};
use crate::args::VcsArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use mcb_utils::constants::vcs::{
    GIT_REF_HEAD, IMPACT_CHANGE_COUNT_WEIGHT, IMPACT_FILE_COUNT_WEIGHT, MAX_IMPACT_SCORE,
};

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
    let repo = match vcs_provider.open_repository(Path::new(&path)).await {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let base_ref = args
        .base_branch
        .clone()
        .unwrap_or_else(|| repo.default_branch().to_owned());
    let head_ref = args
        .target_branch
        .clone()
        .unwrap_or_else(|| GIT_REF_HEAD.to_owned());
    let diff = match vcs_provider.diff_refs(&repo, &base_ref, &head_ref).await {
        Ok(diff) => diff,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let tally = tally_impacted_files(&diff);
    let total_changes = diff.total_additions + diff.total_deletions;
    let impact_score = ((diff.files.len() as f64).ln_1p() * IMPACT_FILE_COUNT_WEIGHT
        + (total_changes as f64).ln_1p() * IMPACT_CHANGE_COUNT_WEIGHT)
        .min(MAX_IMPACT_SCORE);
    let result = ImpactResponse {
        base_ref,
        head_ref,
        impact_score,
        summary: ImpactSummary {
            total_files: diff.files.len(),
            added: tally.added,
            modified: tally.modified,
            deleted: tally.deleted,
            total_changes,
        },
        impacted_files: tally.impacted_files,
    };
    ResponseFormatter::json_success(&result)
}

/// Per-status counts plus the rendered impacted-file list from a ref diff.
struct ImpactTally {
    added: usize,
    modified: usize,
    deleted: usize,
    impacted_files: Vec<ImpactFile>,
}

/// Tally added/modified/deleted counts and build the impacted-file list from a diff.
fn tally_impacted_files(diff: &mcb_domain::entities::vcs::RefDiff) -> ImpactTally {
    let mut tally = ImpactTally {
        added: 0,
        modified: 0,
        deleted: 0,
        impacted_files: Vec::new(),
    };
    for file in &diff.files {
        let status = file.status.to_string();
        match status.as_str() {
            "added" => tally.added += 1,
            "deleted" => tally.deleted += 1,
            _ => tally.modified += 1,
        }
        tally.impacted_files.push(ImpactFile {
            // INTENTIONAL: Path to_str conversion; non-UTF8 paths yield empty string
            path: file.path.to_str().unwrap_or_default().to_owned(),
            status: status.clone(),
            impact: file.additions + file.deletions,
        });
    }
    tally
}
