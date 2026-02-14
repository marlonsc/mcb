use std::process::Command;
use std::sync::OnceLock;
use typed_builder::TypedBuilder;

static PROJECT_CONTEXT: OnceLock<ProjectContext> = OnceLock::new();

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

    /// Resolve project identity from the current git repository.
    ///
    /// Resolution order:
    /// 1. `git config --get remote.origin.url` → parse `owner/repo`
    /// 2. `git rev-parse --show-toplevel` → directory name
    /// 3. `"default"`
    ///
    /// Result is cached for the process lifetime.
    ///
    /// # Architecture Violation (ORG016)
    /// This method implements infrastructure-specific detection logic (Git) within the Domain layer.
    /// According to Clean Architecture principles, the Domain layer should be implementation-agnostic
    /// and contain only business rules and entities. The detection logic should reside in the
    /// Infrastructure or Application layer, with this file defining only the project identity structure.
    ///
    /// TODO(ORG016): Move implementation to application or infrastructure layer.
    /// The Domain layer should remain trait-only for behavioral logic.
    #[must_use]
    pub fn resolve() -> Self {
        PROJECT_CONTEXT.get_or_init(Self::detect).clone()
    }

    /// Detect the current project context using git commands.
    ///
    /// # Architecture Violation (ORG016)
    /// Direct filesystem and external command (git) execution within the Domain layer violates
    /// the boundary between business logic and system infrastructure.
    ///
    /// TODO(ORG016): Extract detection strategy to an Infrastructure service.
    fn detect() -> Self {
        let superproject_id = Self::detect_superproject();
        let is_submodule = superproject_id.is_some();

        if let Some((project_id, project_name)) = Self::from_git_remote() {
            return Self {
                project_id,
                project_name,
                is_submodule,
                superproject_id,
            };
        }
        if let Some((project_id, project_name)) = Self::from_git_toplevel() {
            return Self {
                project_id,
                project_name,
                is_submodule,
                superproject_id,
            };
        }
        Self {
            project_id: "default".to_string(),
            project_name: "default".to_string(),
            is_submodule,
            superproject_id,
        }
    }

    fn from_git_remote() -> Option<(String, String)> {
        let output = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.is_empty() {
            return None;
        }
        let owner_repo = parse_owner_repo(&url)?;
        let name = owner_repo
            .rsplit('/')
            .next()
            .unwrap_or(&owner_repo)
            .to_string();
        Some((owner_repo, name))
    }

    fn from_git_toplevel() -> Option<(String, String)> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let name = std::path::Path::new(&toplevel)
            .file_name()?
            .to_string_lossy()
            .to_string();
        Some((name.clone(), name))
    }

    /// Detect if the current repo is a git submodule.
    /// Returns the superproject's owner/repo if it is.
    ///
    /// # Architecture Violation (ORG016)
    /// Direct git command execution and cross-repository inspection logic within the Domain layer.
    ///
    /// TODO(ORG016): Move cross-repo detection to the Infrastructure/VCS layer.
    fn detect_superproject() -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-superproject-working-tree"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let superproject_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if superproject_path.is_empty() {
            return None;
        }
        // Get the superproject's remote URL
        let output = Command::new("git")
            .args([
                "-C",
                &superproject_path,
                "config",
                "--get",
                "remote.origin.url",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        parse_owner_repo(&url)
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
