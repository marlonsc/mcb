use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

tool_enum! {
/// Actions available for version control system operations
pub enum VcsAction {
    /// List all repositories for the project
    ListRepositories,
    /// Index a specific repository
    IndexRepository,
    /// Compare two branches (diff/impact)
    CompareBranches,
    /// Search for a branch name
    SearchBranch,
    /// Analyze impact of changes
    AnalyzeImpact,
}
}

tool_schema! {
/// Arguments for version control system operations
pub struct VcsArgs {
    /// Action to perform
    #[schemars(
        description = "Action: list_repositories, index_repository, compare_branches, search_branch, analyze_impact"
    )]
    pub action: VcsAction,

    /// Organization identifier
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Repository identifier
    #[schemars(description = "Repository identifier", with = "String")]
    pub repo_id: Option<String>,

    /// Local file system path to the repository
    #[schemars(description = "Repository path on disk", with = "String")]
    #[validate(custom(function = "super::validate_file_path", message = "Invalid file path"))]
    pub repo_path: Option<String>,

    /// Base branch for comparison
    #[schemars(description = "Base branch name", with = "String")]
    pub base_branch: Option<String>,

    /// Target branch for comparison
    #[schemars(description = "Compare/target branch name", with = "String")]
    pub target_branch: Option<String>,

    /// Search query string
    #[schemars(description = "Search query for branch search", with = "String")]
    pub query: Option<String>,

    /// Specific branches to index
    #[schemars(
        description = "Branches to index (default: repo default branch)",
        with = "Vec<String>"
    )]
    pub branches: Option<Vec<String>>,

    /// Whether to index commit history
    #[schemars(
        description = "Whether to include commit history when indexing",
        with = "bool"
    )]
    pub include_commits: Option<bool>,

    /// Depth of history to index
    #[schemars(
        description = "Commit history depth (default: 50 from config, or 1000 if no config)",
        with = "usize"
    )]
    pub depth: Option<usize>,

    /// Result limit for list/search operations
    #[schemars(description = "Limit for search or list actions", with = "u32")]
    pub limit: Option<u32>,
}
}
