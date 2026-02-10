use mcb_domain::entities::vcs::RepositoryId;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::CollectionId;
use std::path::PathBuf;

/// Temporary compatibility function for mapping collection names.
/// Replaces the old JSON-based mapping until Phase 2 implements the full CollectionRepository.
pub fn map_collection_name(user_name: &str) -> Result<CollectionId> {
    // For now, just normalize and return.
    // In Phase 2, this will check the DB or be replaced by the service.
    let normalized = user_name.replace('-', "_").to_lowercase();
    Ok(CollectionId::new(normalized))
}

// VCS Registry Compatibility Stubs
// These replace the old JSON registry until Phase 2.3 implements VcsPersistenceRepository.

pub fn record_repository(_repository_id: &RepositoryId, _path: &std::path::Path) -> Result<()> {
    // No-op for now. New implementation will persist to SQLite.
    Ok(())
}

pub fn lookup_repository_path(_repository_id: &RepositoryId) -> Result<PathBuf> {
    // Return error as we don't have the registry anymore.
    // In Phase 2.3 this will query SQLite.
    Err(Error::repository_not_found(
        "Legacy registry removed. Waiting for Phase 2.3 implementation.",
    ))
}

pub fn list_repositories() -> Result<Vec<(RepositoryId, PathBuf)>> {
    Ok(vec![])
}
