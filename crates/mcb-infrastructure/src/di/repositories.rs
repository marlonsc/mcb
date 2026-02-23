//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! Repository factories for standalone/server composition.
//!
//! Provides default repository implementations so that the server layer does not
//! import concrete providers directly (CA006).
//!
//! Consumer crates (mcb-server tests, golden tests, etc.) MUST use these
//! wrappers instead of importing from `mcb-providers` directly.

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    DatabaseExecutor, DatabaseProvider, MemoryRepository, VcsEntityRepository,
};
use mcb_domain::registry::database::{
    DatabaseProviderConfig, list_database_providers, resolve_database_provider,
};

fn resolve_default_database_provider() -> Result<Arc<dyn DatabaseProvider>> {
    let provider_name = list_database_providers()
        .first()
        .map(|(name, _)| *name)
        .ok_or_else(|| {
            Error::configuration("Database: no providers registered in linkme registry")
        })?;

    resolve_database_provider(&DatabaseProviderConfig::new(provider_name))
}

async fn connect_executor(path: PathBuf) -> Result<Arc<dyn DatabaseExecutor>> {
    let provider = resolve_default_database_provider()?;
    provider.connect(path.as_path()).await
}

/// Creates a VCS entity repository backed by the provided database executor.
///
/// Centralizes repository instantiation in the infrastructure layer.
///
/// # Errors
///
/// Returns an error if no database provider is registered or resolution fails.
pub fn create_vcs_entity_repository(
    executor: Arc<dyn DatabaseExecutor>,
) -> Result<Arc<dyn VcsEntityRepository>> {
    let provider = resolve_default_database_provider()?;
    Ok(provider.create_vcs_entity_repository(executor))
}

/// Create a file-backed memory repository.
///
/// Wraps the provider factory so consumer crates never import `mcb_providers`.
///
/// # Errors
///
/// Returns an error if the database connection or schema initialization fails.
pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    let provider = resolve_default_database_provider()?;
    let executor = connect_executor(path).await?;
    Ok(provider.create_memory_repository(executor))
}

/// Create a file-backed memory repository and its database executor.
///
/// Wraps the provider factory so consumer crates never import `mcb_providers`.
///
/// # Errors
///
/// Returns an error if the database connection or schema initialization fails.
pub async fn create_memory_repository_with_executor(
    path: PathBuf,
) -> Result<(Arc<dyn MemoryRepository>, Arc<dyn DatabaseExecutor>)> {
    let provider = resolve_default_database_provider()?;
    let executor = connect_executor(path).await?;
    let memory_repository = provider.create_memory_repository(Arc::clone(&executor));
    Ok((memory_repository, executor))
}
