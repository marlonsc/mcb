//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type of version control system.
crate::define_string_enum! {
    /// Type of version control system.
    pub enum VcsType [strum = "snake_case", serde = "snake_case", schema] {
        /// Git repository.
        #[strum(serialize = "git")]
        Git,
        /// Mercurial repository.
        #[strum(serialize = "mercurial", serialize = "hg")]
        Mercurial,
        /// Subversion repository.
        #[strum(serialize = "svn", serialize = "subversion")]
        Svn,
    }
}

impl Copy for VcsType {}

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

crate::define_entity! {
    /// A tracked repository registered in the platform.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Repository { id, org_id, project_id, created_at, updated_at } {
        /// Display name of the repository.
        pub name: String,
        /// Remote URL of the repository (e.g. <https://github.com/user/repo.git>).
        pub url: String,
        /// Local path where the repository is checked out.
        pub local_path: String,
        /// Type of version control system used.
        pub vcs_type: VcsType,
    }
}

// ---------------------------------------------------------------------------
// Branch
// ---------------------------------------------------------------------------

crate::define_entity! {
    /// A tracked branch within a repository.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Branch { id, org_id, created_at } {
        /// Repository this branch belongs to.
        pub repository_id: String,
        /// Branch name (e.g. "main", "feat/data-model-v2").
        pub name: String,
        /// Whether this is the repository's default branch.
        pub is_default: bool,
        /// Current HEAD commit SHA.
        pub head_commit: String,
        /// Upstream tracking branch (e.g. "origin/main").
        pub upstream: Option<String>,
    }
}
