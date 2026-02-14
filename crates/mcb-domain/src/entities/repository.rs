//! Repository Domain Entities
//!
//! This module defines entities for tracking and managing Version Control System (VCS)
//! repositories. It facilitates multi-tenant environment support by associating
//! repositories with Organizations and Projects.
//!
//! # Core Entities
//! - [`Repository`]: A persisted record of a remote or local VCS repository (Git, Hg, SVN).
//! - [`Branch`]: A specific line of development within a repository.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// A tracked VCS repository belonging to a project within an organization.
///
/// # Code Smells
/// TODO(qlty): Found 20 lines of similar code with `crates/mcb-domain/src/entities/plan.rs`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Repository {
    /// Unique identifier (UUID).
    pub id: String,
    /// Organization that owns this repository.
    pub org_id: String,
    /// Project this repository belongs to.
    pub project_id: String,
    /// Human-readable display name (e.g. "mcb-data-model-v2").
    pub name: String,
    /// Remote URL (e.g. `https://github.com/org/repo`).
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
    pub fn as_str(&self) -> &str {
        match self {
            Self::Git => "git",
            Self::Mercurial => "mercurial",
            Self::Svn => "svn",
        }
    }
}

// ---------------------------------------------------------------------------
// Branch
// ---------------------------------------------------------------------------

/// A tracked branch within a repository.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
