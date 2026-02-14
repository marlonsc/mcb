use typed_builder::TypedBuilder;

/// Auto-resolved project identity derived from the git repository.
///
/// Without authentication the project is identified by `owner/repo`
/// parsed from `git remote.origin.url`.  This is stable across
/// worktrees, directory renames, and multiple checkouts of the same
/// repository.
#[derive(Debug, Clone, TypedBuilder)]
pub struct ProjectContext {
    /// Stable project identifier (e.g. `"marlonsc/mcb"`).
    #[builder(setter(into))]
    pub project_id: String,
    /// Human-readable short name (e.g. `"mcb"`).
    #[builder(setter(into))]
    pub project_name: String,
    /// Whether this repository is a git submodule.
    #[builder(default)]
    pub is_submodule: bool,
    /// The superproject's owner/repo if this is a submodule.
    #[builder(default, setter(strip_option, into))]
    pub superproject_id: Option<String>,
}

impl ProjectContext {
    /// Explicit constructor for tests and overrides.
    pub fn new(project_id: impl Into<String>, project_name: impl Into<String>) -> Self {
        Self::builder()
            .project_id(project_id.into())
            .project_name(project_name.into())
            .build()
    }
}

/// Parse a git remote URL into `owner/repo` format.
///
/// Supports SSH shorthand (`git@host:owner/repo.git`),
/// SSH URL (`ssh://git@host/owner/repo.git`),
/// and HTTPS (`https://host/owner/repo[.git]`).
pub fn parse_owner_repo(url: &str) -> Option<String> {
    // SSH shorthand: git@host:owner/repo.git
    if let Some((_host, path)) = url.strip_prefix("git@").and_then(|s| s.split_once(':')) {
        let path = path.trim_end_matches(".git");
        return normalize_owner_repo(path);
    }

    // URL form: scheme://host/owner/repo.git
    let path = url
        .split("://")
        .nth(1)
        .and_then(|s| s.split_once('/'))
        .map(|(_, path)| path)?;
    let path = path.trim_end_matches(".git");
    normalize_owner_repo(path)
}

/// Normalize a repository path into `owner/repo` or `org/subgroup/repo`.
///
/// Returns `None` when the input path does not contain at least one
/// non-empty segment.
pub fn normalize_owner_repo(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        Some(parts.join("/"))
    } else if parts.len() == 1 {
        Some(parts[0].to_string())
    } else {
        None
    }
}
