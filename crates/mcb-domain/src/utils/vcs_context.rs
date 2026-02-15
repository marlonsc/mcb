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
