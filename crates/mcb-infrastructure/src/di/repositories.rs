//! Repository factories for standalone/server composition.
//!
//! Provides default repository implementations so that the server layer does not
//! import concrete providers directly (CA006).
//!
//! Consumer crates (mcb-server tests, golden tests, etc.) MUST use these
//! wrappers instead of importing from `mcb-providers` directly.

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::{DatabaseExecutor, MemoryRepository, VcsEntityRepository};
use mcb_providers::database;

/// Creates a VCS entity repository backed by the provided database executor.
///
/// Centralizes repository instantiation in the infrastructure layer.
pub fn create_vcs_entity_repository(
    executor: Arc<dyn DatabaseExecutor>,
) -> Arc<dyn VcsEntityRepository> {
    database::create_vcs_entity_repository_from_executor(executor)
}

/// Create a file-backed memory repository.
///
/// Wraps the provider factory so consumer crates never import `mcb_providers`.
///
/// # Errors
///
/// Returns an error if the database connection or schema initialization fails.
pub async fn create_memory_repository(path: PathBuf) -> Result<Arc<dyn MemoryRepository>> {
    database::create_memory_repository(path).await
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
    database::create_memory_repository_with_executor(path).await
}
