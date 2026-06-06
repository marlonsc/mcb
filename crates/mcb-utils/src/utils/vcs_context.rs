//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
/// VCS context (branch, commit, repo id) captured from the current environment.
#[derive(Clone)]
pub struct VcsContext {
    /// Stores the branch value.
    pub branch: Option<String>,
    /// Stores the commit value.
    pub commit: Option<String>,
    /// Stores the repo id value.
    pub repo_id: Option<String>,
}

impl VcsContext {
    /// Creates a new `VcsContext` from pre-resolved values.
    #[must_use]
    pub fn new(branch: Option<String>, commit: Option<String>, repo_id: Option<String>) -> Self {
        Self {
            branch,
            commit,
            repo_id,
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime VCS Context Capture
// ---------------------------------------------------------------------------

use std::process::Command;
use std::sync::OnceLock;

use crate::constants::vcs;

static VCS_CONTEXT: OnceLock<VcsContext> = OnceLock::new();

/// Capture VCS context (branch, commit, repo) from the git environment.
///
/// Result is cached after the first call via `OnceLock`.
#[must_use]
pub fn capture_vcs_context() -> VcsContext {
    VCS_CONTEXT
        .get_or_init(|| {
            let git_output = |args: &[&str]| {
                Command::new(vcs::GIT_COMMAND)
                    .args(args)
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            Some(String::from_utf8_lossy(&o.stdout).trim().to_owned())
                        } else {
                            None
                        }
                    })
            };

            let branch = git_output(&["rev-parse", "--abbrev-ref", vcs::GIT_REF_HEAD]);
            let commit = git_output(&["rev-parse", vcs::GIT_REF_HEAD]);
            let repo_id = git_output(&["config", "--get", "remote.origin.url"]);

            VcsContext::new(branch, commit, repo_id)
        })
        .clone()
}
