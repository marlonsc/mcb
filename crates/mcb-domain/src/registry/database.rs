//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Database Repository Registry
//!
//! Auto-registration system for database repository bundles using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

use crate::ports::{
    AgentRepository, AuthRepositoryPort, DashboardQueryPort, FileHashRepository,
    IssueEntityRepository, MemoryRepository, OrgEntityRepository, PlanEntityRepository,
    ProjectRepository, VcsEntityRepository,
};

// ---------------------------------------------------------------------------
// Database connection provider (factory for opaque DB connections)
// ---------------------------------------------------------------------------

/// Configuration for resolving a database connection provider.
#[derive(Debug, Clone)]
pub struct DatabaseProviderConfig {
    /// Provider name (e.g. "sqlite", "postgres").
    pub provider: String,
    /// Optional path for file-based databases like SQLite.
    pub path: Option<PathBuf>,
}

impl DatabaseProviderConfig {
    /// Create a new config with only a provider name.
    #[must_use]
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            path: None,
        }
    }

    /// Set the database file path (for SQLite etc.).
    #[must_use]
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

/// Registry entry for a database connection provider.
pub struct DatabaseConnectionEntry {
    /// Unique provider name.
    pub name: &'static str,
    /// Factory that builds a connection and runs migrations.
    /// Returns an opaque `Arc<dyn Any + Send + Sync>` wrapping the connection.
    pub build: fn(
        &DatabaseProviderConfig,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::error::Result<Arc<dyn Any + Send + Sync>>>
                + Send,
        >,
    >,
}

#[linkme::distributed_slice]
/// Registered database connection providers.
pub static DATABASE_CONNECTION_PROVIDERS: [DatabaseConnectionEntry] = [..];

/// Resolve a database connection provider by config and build a connection.
///
/// # Errors
///
/// Returns an error when the provider name is unknown or when connection fails.
pub async fn resolve_database_provider(
    config: &DatabaseProviderConfig,
) -> crate::error::Result<Arc<dyn Any + Send + Sync>> {
    for entry in DATABASE_CONNECTION_PROVIDERS.iter() {
        if entry.name == config.provider {
            return (entry.build)(config).await;
        }
    }

    let available: Vec<&str> = DATABASE_CONNECTION_PROVIDERS
        .iter()
        .map(|entry| entry.name)
        .collect();

    Err(crate::error::Error::configuration(format!(
        "Unknown database connection provider '{}'. Available: {available:?}",
        config.provider,
    )))
}

// ---------------------------------------------------------------------------
// Database repository bundle provider
// ---------------------------------------------------------------------------

/// Complete set of database-backed repositories required by the application.
pub struct DatabaseRepositories {
    /// Repository for memory entities.
    pub memory: Arc<dyn MemoryRepository>,
    /// Repository for authentication data.
    pub auth: Arc<dyn AuthRepositoryPort>,
    /// Repository for dashboard queries.
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
    pub build: fn(Arc<dyn Any + Send + Sync>, String) -> crate::error::Result<DatabaseRepositories>,
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
    connection: Arc<dyn Any + Send + Sync>,
    namespace: String,
) -> crate::error::Result<DatabaseRepositories> {
    for entry in DATABASE_REPOSITORY_PROVIDERS {
        if entry.name == provider_name {
            return (entry.build)(connection, namespace);
        }
    }

    let available: Vec<&str> = DATABASE_REPOSITORY_PROVIDERS
        .iter()
        .map(|entry| entry.name)
        .collect();

    Err(crate::error::Error::configuration(format!(
        "Unknown database repository provider '{provider_name}'. Available providers: {available:?}"
    )))
}

/// List all registered database repository providers.
#[must_use]
pub fn list_database_providers() -> Vec<(&'static str, &'static str)> {
    DATABASE_REPOSITORY_PROVIDERS
        .iter()
        .map(|entry| (entry.name, entry.description))
        .collect()
}

// ---------------------------------------------------------------------------
// Database migration provider (linkme registry using MigrationProvider port)
// ---------------------------------------------------------------------------

/// Registry entry for a database migration provider.
///
/// Each provider registers a factory function that builds an
/// `Arc<dyn MigrationProvider>`.  Domain consumers call
/// [`resolve_migration_provider`] to resolve the first registered provider.
pub struct MigrationProviderEntry {
    /// Unique provider name (e.g. "seaorm").
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Factory that builds a migration provider instance.
    pub build: fn() -> Arc<dyn crate::ports::MigrationProvider>,
}

#[linkme::distributed_slice]
/// Registered database migration providers.
pub static MIGRATION_PROVIDERS: [MigrationProviderEntry] = [..];

/// Resolve the first registered migration provider.
///
/// # Errors
///
/// Returns an error when no migration provider has been registered.
pub fn resolve_migration_provider() -> crate::error::Result<Arc<dyn crate::ports::MigrationProvider>>
{
    if let Some(entry) = MIGRATION_PROVIDERS.iter().next() {
        return Ok((entry.build)());
    }
    Err(crate::error::Error::configuration(
        "No migration provider registered. Ensure mcb-providers is linked.",
    ))
}

/// List all registered migration providers.
#[must_use]
pub fn list_migration_providers() -> Vec<(&'static str, &'static str)> {
    MIGRATION_PROVIDERS
        .iter()
        .map(|entry| (entry.name, entry.description))
        .collect()
}

/// Apply all pending migrations (up) using the DI-resolved provider.
///
/// The `db` parameter is a type-erased database connection
/// (e.g. `Box<DatabaseConnection>` from SeaORM).
///
/// `steps` limits how many pending migrations to apply; `None` applies all.
///
/// # Errors
///
/// Returns an error when no migration provider has been registered or
/// when the underlying migration fails.
pub async fn migrate_up(
    db: Box<dyn std::any::Any + Send + Sync>,
    steps: Option<u32>,
) -> crate::error::Result<()> {
    let provider = resolve_migration_provider()?;
    provider.migrate_up(db, steps).await
}

/// Rollback applied migrations using the DI-resolved provider.
///
/// `steps` limits how many migrations to rollback; `None` rolls back all.
///
/// # Errors
///
/// Returns an error when no migration provider has been registered or
/// when the underlying rollback fails.
pub async fn migrate_down(
    db: Box<dyn std::any::Any + Send + Sync>,
    steps: Option<u32>,
) -> crate::error::Result<()> {
    let provider = resolve_migration_provider()?;
    provider.migrate_down(db, steps).await
}
