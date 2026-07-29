//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

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

// ---------------------------------------------------------------------------
// MCP-facing single-purpose tools
// ---------------------------------------------------------------------------

tool_action! {
    /// Arguments for the `list_repos` tool.
    pub struct ListReposArgs => VcsArgs {
        #[schemars(description = "Maximum results", with = "u32")]
        limit: Option<u32>
        ;
        hidden {
            org_id: Option<String>, repo_id: Option<String>,
            repo_path: Option<String>,
        }
        ;
        convert |a| {
            action: VcsAction::ListRepositories, base_branch: None, target_branch: None,
            query: None, branches: None, include_commits: None, depth: None, limit: a.limit,
        }
    }
}

tool_action! {
    /// Arguments for the `compare_branches` tool.
    pub struct CompareBranchesArgs => VcsArgs {
        #[schemars(description = "Base branch name")]
        base_branch: String,
        #[schemars(description = "Target branch name")]
        target_branch: String,
        #[schemars(description = "Include commit history", with = "bool")]
        include_commits: Option<bool>,
        #[schemars(description = "Commit history depth (default: 50)", with = "usize")]
        depth: Option<usize>
        ;
        hidden {
            org_id: Option<String>, repo_id: Option<String>,
            repo_path: Option<String>,
        }
        ;
        convert |a| {
            action: VcsAction::CompareBranches,
            base_branch: Some(a.base_branch), target_branch: Some(a.target_branch),
            query: None, branches: None,
            include_commits: a.include_commits, depth: a.depth, limit: None,
        }
    }
}

tool_action! {
    /// Arguments for the `analyze_impact` tool.
    pub struct AnalyzeImpactArgs => VcsArgs {
        #[schemars(description = "Branches to analyze", with = "Vec<String>")]
        branches: Option<Vec<String>>,
        #[schemars(description = "Analysis depth (default: 1000)", with = "usize")]
        depth: Option<usize>,
        #[schemars(description = "Maximum results", with = "u32")]
        limit: Option<u32>
        ;
        hidden {
            org_id: Option<String>, repo_id: Option<String>,
            repo_path: Option<String>,
        }
        ;
        convert |a| {
            action: VcsAction::AnalyzeImpact, base_branch: None, target_branch: None,
            query: None, branches: a.branches, include_commits: None,
            depth: a.depth, limit: a.limit,
        }
    }
}
