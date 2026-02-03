//! VCS context capture for memory observations (MEM-06).
//! Captures branch, commit, and repo id from the current environment (e.g. git).

use std::process::Command;

/// VCS context (branch, commit, repo id) captured from the current environment.
pub struct VcsContext {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub repo_id: Option<String>,
}

impl VcsContext {
    /// Capture VCS context from current environment (e.g. git).
    #[must_use]
    pub fn capture() -> Self {
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });
        let commit = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });
        let repo_id = Command::new("git")
            .args(["rev-parse", "--verify", "HEAD"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });
        Self {
            branch,
            commit,
            repo_id,
        }
    }
}
