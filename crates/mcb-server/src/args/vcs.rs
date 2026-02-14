use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

tool_enum! {
pub enum VcsAction {
    ListRepositories,
    IndexRepository,
    CompareBranches,
    SearchBranch,
    AnalyzeImpact,
}
}

tool_schema! {
pub struct VcsArgs {
    #[schemars(
        description = "Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact"
    )]
    pub action: VcsAction,

    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    #[schemars(description = "Repository identifier", with = "String")]
    pub repo_id: Option<String>,

    #[schemars(description = "Repository path on disk", with = "String")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub repo_path: Option<String>,

    #[schemars(description = "Base branch name", with = "String")]
    pub base_branch: Option<String>,

    #[schemars(description = "Compare/target branch name", with = "String")]
    pub target_branch: Option<String>,

    #[schemars(description = "Search query for branch search", with = "String")]
    pub query: Option<String>,

    #[schemars(
        description = "Branches to index (default: repo default branch)",
        with = "Vec<String>"
    )]
    pub branches: Option<Vec<String>>,

    #[schemars(
        description = "Whether to include commit history when indexing",
        with = "bool"
    )]
    pub include_commits: Option<bool>,

    #[schemars(
        description = "Commit history depth (default: 50 from config, or 1000 if no config)",
        with = "usize"
    )]
    pub depth: Option<usize>,

    #[schemars(description = "Limit for search or list actions", with = "u32")]
    pub limit: Option<u32>,
}
}
