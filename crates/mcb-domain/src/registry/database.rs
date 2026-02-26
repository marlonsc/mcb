//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Database Repository Registry
//!
//! Auto-registration system for database repository bundles using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::any::Any;
use std::sync::Arc;

use crate::ports::{
    AgentRepository, AuthRepositoryPort, DashboardQueryPort, FileHashRepository,
    IssueEntityRepository, MemoryRepository, OrgEntityRepository, PlanEntityRepository,
    ProjectRepository, VcsEntityRepository,
};

/// Complete set of database-backed repositories required by the application.
pub struct DatabaseRepositories {
    /// Repository for memory entities.
    pub memory: Arc<dyn MemoryRepository>,
    pub auth: Arc<dyn AuthRepositoryPort>,
    pub dashboard: Arc<dyn DashboardQueryPort>,
    /// Repository for agent entities.
    pub agent: Arc<dyn AgentRepository>,
    /// Repository for project entities.
    pub project: Arc<dyn ProjectRepository>,
    /// Repository for VCS entities.
    pub vcs_entity: Arc<dyn VcsEntityRepository>,
    /// Repository for plan entities.
    pub plan_entity: Arc<dyn PlanEntityRepository>,
    /// Repository for issue entities.
    pub issue_entity: Arc<dyn IssueEntityRepository>,
    /// Repository for organization entities.
    pub org_entity: Arc<dyn OrgEntityRepository>,
    /// Repository for file hash entities.
    pub file_hash: Arc<dyn FileHashRepository>,
}

/// Registry entry for a database repository provider.
pub struct DatabaseRepositoryEntry {
    /// Unique provider name.
    pub name: &'static str,
    /// Human-readable provider description.
    pub description: &'static str,
    /// Factory that builds a full repository bundle.
    pub build: fn(Box<dyn Any + Send + Sync>, String) -> Result<DatabaseRepositories, String>,
}

#[linkme::distributed_slice]
/// Registered database repository providers.
pub static DATABASE_REPOSITORY_PROVIDERS: [DatabaseRepositoryEntry] = [..];

/// Resolve a database repository provider by name and build repositories.
///
/// # Errors
///
/// Returns an error when the provider name is unknown or when the selected
/// provider fails to construct repository instances.
pub fn resolve_database_repositories(
    provider_name: &str,
    connection: Box<dyn Any + Send + Sync>,
    namespace: String,
) -> Result<DatabaseRepositories, String> {
    for entry in DATABASE_REPOSITORY_PROVIDERS {
        if entry.name == provider_name {
            return (entry.build)(connection, namespace);
        }
    }

    let available: Vec<&str> = DATABASE_REPOSITORY_PROVIDERS
        .iter()
        .map(|entry| entry.name)
        .collect();

    Err(format!(
        "Unknown database repository provider '{provider_name}'. Available providers: {available:?}"
    ))
}

/// List all registered database repository providers.
#[must_use]
pub fn list_database_providers() -> Vec<(&'static str, &'static str)> {
    DATABASE_REPOSITORY_PROVIDERS
        .iter()
        .map(|entry| (entry.name, entry.description))
        .collect()
}
