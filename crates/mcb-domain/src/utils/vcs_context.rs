//! VCS context capture for memory observations (MEM-06).
//! Captures branch, commit, and repo id from the current environment (e.g. git).

use std::process::Command;
use std::sync::OnceLock;

/// VCS context (branch, commit, repo id) captured from the current environment.
#[derive(Clone)]
pub struct VcsContext {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub repo_id: Option<String>,
}

static VCS_CONTEXT: OnceLock<VcsContext> = OnceLock::new();

impl VcsContext {
    /// Capture VCS context from current environment (e.g. git).
    ///
    /// Results are cached to avoid repeated git process spawning.
    /// Optimized to batch git commands and reduce process spawning overhead.
    #[must_use]
    pub fn capture() -> Self {
        VCS_CONTEXT
            .get_or_init(|| {
                // Batch branch and commit lookup into a single git rev-parse invocation
                let (branch, commit) = Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD", "HEAD"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            let output = String::from_utf8_lossy(&o.stdout);
                            let mut lines = output.lines();
                            let branch = lines.next().map(|s| s.trim().to_string());
                            let commit = lines.next().map(|s| s.trim().to_string());
                            Some((branch, commit))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((None, None));

                let repo_id = Command::new("git")
                    .args(["config", "--get", "remote.origin.url"])
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
            })
            .clone()
    }
}
