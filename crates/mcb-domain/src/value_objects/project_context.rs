use std::process::Command;
use std::sync::OnceLock;

static PROJECT_CONTEXT: OnceLock<ProjectContext> = OnceLock::new();

/// Auto-resolved project identity derived from the git repository.
///
/// Without authentication the project is identified by `owner/repo`
/// parsed from `git remote.origin.url`.  This is stable across
/// worktrees, directory renames, and multiple checkouts of the same
/// repository.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// Stable project identifier (e.g. `"marlonsc/mcb"`).
    pub project_id: String,
    /// Human-readable short name (e.g. `"mcb"`).
    pub project_name: String,
}

impl ProjectContext {
    /// Explicit constructor for tests and overrides.
    pub fn new(project_id: impl Into<String>, project_name: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            project_name: project_name.into(),
        }
    }

    /// Resolve project identity from the current git repository.
    ///
    /// Resolution order:
    /// 1. `git config --get remote.origin.url` → parse `owner/repo`
    /// 2. `git rev-parse --show-toplevel` → directory name
    /// 3. `"default"`
    ///
    /// Result is cached for the process lifetime.
    #[must_use]
    pub fn resolve() -> Self {
        PROJECT_CONTEXT.get_or_init(Self::detect).clone()
    }

    fn detect() -> Self {
        if let Some(ctx) = Self::from_git_remote() {
            return ctx;
        }
        if let Some(ctx) = Self::from_git_toplevel() {
            return ctx;
        }
        Self {
            project_id: "default".to_string(),
            project_name: "default".to_string(),
        }
    }

    fn from_git_remote() -> Option<Self> {
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
        let owner_repo = parse_remote_url(&url)?;
        let name = owner_repo
            .rsplit('/')
            .next()
            .unwrap_or(&owner_repo)
            .to_string();
        Some(Self {
            project_id: owner_repo,
            project_name: name,
        })
    }

    fn from_git_toplevel() -> Option<Self> {
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
        Some(Self {
            project_id: name.clone(),
            project_name: name,
        })
    }
}

/// Parse a git remote URL into `owner/repo` format.
///
/// Supports SSH shorthand (`git@host:owner/repo.git`),
/// SSH URL (`ssh://git@host/owner/repo.git`),
/// and HTTPS (`https://host/owner/repo[.git]`).
pub fn parse_remote_url(url: &str) -> Option<String> {
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

fn normalize_owner_repo(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else if parts.len() == 1 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh_shorthand() {
        assert_eq!(
            parse_remote_url("git@github.com:marlonsc/mcb.git"),
            Some("marlonsc/mcb".to_string())
        );
    }

    #[test]
    fn parse_https() {
        assert_eq!(
            parse_remote_url("https://github.com/marlonsc/mcb.git"),
            Some("marlonsc/mcb".to_string())
        );
    }

    #[test]
    fn parse_https_no_suffix() {
        assert_eq!(
            parse_remote_url("https://github.com/marlonsc/mcb"),
            Some("marlonsc/mcb".to_string())
        );
    }

    #[test]
    fn parse_ssh_url() {
        assert_eq!(
            parse_remote_url("ssh://git@github.com/marlonsc/mcb.git"),
            Some("marlonsc/mcb".to_string())
        );
    }

    #[test]
    fn parse_gitlab_subgroup() {
        assert_eq!(
            parse_remote_url("git@gitlab.com:org/subgroup/repo.git"),
            Some("org/subgroup".to_string())
        );
    }

    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(parse_remote_url(""), None);
    }

    #[test]
    fn resolve_returns_consistent_value() {
        let ctx1 = ProjectContext::resolve();
        let ctx2 = ProjectContext::resolve();
        assert_eq!(ctx1.project_id, ctx2.project_id);
    }
}
