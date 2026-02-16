//! Runtime context value objects for tenant isolation and project identity.

/// Organization tenant context
pub mod org;
/// Project identity auto-resolved from git repository
pub mod project;
/// Project configuration overrides from `.mcb/project.toml`
pub mod project_settings;
/// VCS context (branch, commit, repo id) from the current environment
pub mod vcs;

pub use org::OrgContext;
pub use project::{ProjectContext, normalize_owner_repo, parse_owner_repo};
pub use project_settings::{
    ProjectEmbeddingConfig, ProjectProvidersSettings, ProjectSettings, ProjectVectorStoreConfig,
};
pub use vcs::VcsContext;
