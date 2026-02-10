//! Persisted VCS repository and branch entities for multi-tenant CRUD.
//!
//! These are the *persisted* counterparts of the read-only [`VcsRepository`]
//! and [`VcsBranch`] models used by the VCS provider.  They carry `org_id`
//! and timestamps for row-level tenant isolation.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// A tracked VCS repository belonging to a project within an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique identifier (UUID).
    pub id: String,
    /// Organization that owns this repository.
    pub org_id: String,
    /// Project this repository belongs to.
    pub project_id: String,
    /// Human-readable display name (e.g. "mcb-data-model-v2").
    pub name: String,
    /// Remote URL (e.g. "https://github.com/org/repo").
    pub url: String,
    /// Local filesystem path where the repo is cloned.
    pub local_path: String,
    /// Version control system type.
    pub vcs_type: VcsType,
    /// Timestamp when the repository was first tracked (Unix epoch).
    pub created_at: i64,
    /// Timestamp of last metadata update (Unix epoch).
    pub updated_at: i64,
}

/// Type of version control system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VcsType {
    /// Git repository.
    Git,
    /// Mercurial repository.
    Mercurial,
    /// Subversion repository.
    Svn,
}

impl VcsType {
    /// Returns the string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Mercurial => "mercurial",
            Self::Svn => "svn",
        }
    }
}

impl std::fmt::Display for VcsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for VcsType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "mercurial" | "hg" => Ok(Self::Mercurial),
            "svn" | "subversion" => Ok(Self::Svn),
            _ => Err(format!("Unknown VCS type: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Branch
// ---------------------------------------------------------------------------

/// A tracked branch within a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Unique identifier (UUID).
    pub id: String,
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
    /// Timestamp when the branch was first tracked (Unix epoch).
    pub created_at: i64,
}
