//! Repository factories for standalone/server composition.
//!
//! Provides default repository implementations so that the server layer does not
//! import concrete providers directly (CA006).

use std::sync::Arc;

use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::repositories::VcsEntityRepository;
use mcb_providers::database;

/// Creates a VCS entity repository backed by the provided database executor.
///
/// Centralizes repository instantiation in the infrastructure layer.
pub fn create_vcs_entity_repository(
    executor: Arc<dyn DatabaseExecutor>,
) -> Arc<dyn VcsEntityRepository> {
    database::create_vcs_entity_repository_from_executor(executor)
}
