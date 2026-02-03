//! Handler for the `analyze_impact` MCP tool

use crate::args::AnalyzeImpactArgs;
use mcb_domain::entities::vcs::DiffStatus;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use validator::Validate;

pub struct AnalyzeImpactHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

#[derive(Serialize)]
struct ImpactAnalysis {
    base_ref: String,
    head_ref: String,
    impact_score: f64,
    summary: ImpactSummary,
    impacted_files: Vec<ImpactedFile>,
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
struct ImpactedFile {
    path: String,
    status: String,
    impact: String,
}

impl AnalyzeImpactHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<AnalyzeImpactArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let path = Path::new(&args.path);

        let repo = match self.vcs_provider.open_repository(path).await {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to open repository: {e}"
                ))]));
            }
        };

        let diff = match self
            .vcs_provider
            .diff_refs(&repo, &args.base_ref, &args.head_ref)
            .await
        {
            Ok(d) => d,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to analyze impact: {e}"
                ))]));
            }
        };

        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;

        let impacted_files: Vec<ImpactedFile> = diff
            .files
            .iter()
            .map(|f| {
                let (status_str, impact) = match f.status {
                    DiffStatus::Added => {
                        added += 1;
                        ("Added", "New file")
                    }
                    DiffStatus::Modified => {
                        modified += 1;
                        ("Modified", "Changed")
                    }
                    DiffStatus::Deleted => {
                        deleted += 1;
                        ("Deleted", "Removed")
                    }
                    DiffStatus::Renamed => {
                        modified += 1;
                        ("Renamed", "Moved")
                    }
                };
                ImpactedFile {
                    path: f.path.to_string_lossy().to_string(),
                    status: status_str.to_string(),
                    impact: impact.to_string(),
                }
            })
            .collect();

        let total_changes = diff.total_additions + diff.total_deletions;
        let impact_score = Self::calculate_impact_score(diff.files.len(), total_changes);

        let result = ImpactAnalysis {
            base_ref: args.base_ref,
            head_ref: args.head_ref,
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

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string());

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    fn calculate_impact_score(files_changed: usize, total_changes: usize) -> f64 {
        let file_weight = (files_changed as f64).ln_1p() * 10.0;
        let change_weight = (total_changes as f64).ln_1p() * 5.0;
        (file_weight + change_weight).min(100.0)
    }
}
