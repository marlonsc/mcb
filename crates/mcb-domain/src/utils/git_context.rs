//! Git context capture for memory observations (MEM-06)

use std::process::Command;

pub struct GitContext {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub repo_id: Option<String>,
}

impl GitContext {
    /// Capture git context from current environment
    pub fn capture() -> Self {
        let branch = Self::get_branch();
        let commit = Self::get_commit();
        let repo_id = Self::get_repo_id();

        Self {
            branch,
            commit,
            repo_id,
        }
    }

    fn get_branch() -> Option<String> {
        Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn get_commit() -> Option<String> {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn get_repo_id() -> Option<String> {
        Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Apply git context to observation metadata
    pub fn apply_to_metadata(&self, metadata: &mut crate::entities::memory::ObservationMetadata) {
        if let Some(ref branch) = self.branch {
            metadata.branch = Some(branch.clone());
        }
        if let Some(ref commit) = self.commit {
            metadata.commit = Some(commit.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_context_fields_optional() {
        let ctx = GitContext {
            branch: Some("main".to_string()),
            commit: Some("abc123".to_string()),
            repo_id: Some("repo123".to_string()),
        };

        assert_eq!(ctx.branch, Some("main".to_string()));
        assert_eq!(ctx.commit, Some("abc123".to_string()));
    }
}
