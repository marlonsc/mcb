//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type of version control system.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum VcsType {
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

impl VcsType {
    /// Returns the string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Git => "git",
            Self::Mercurial => "mercurial",
            Self::Svn => "svn",
        }
    }
}

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

crate::define_entity_org_project_audited! {
    /// A tracked repository registered in the platform.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Repository {
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

crate::define_entity_org_created! {
    /// A tracked branch within a repository.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Branch {
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
