//! Test utilities for mcb-server
//!
//! Provides mock implementations of domain service interfaces and test fixtures
//! for handler testing.

pub mod helpers;
pub mod mock_services;
pub mod real_providers;
pub mod test_fixtures;

use mcb_application::services::RepositoryResolver;
use mcb_domain::ports::repositories::VcsEntityRepository;
use mcb_domain::value_objects::project_context::ProjectContext;
use std::sync::Arc;

#[allow(dead_code)]
pub fn test_resolver() -> Arc<RepositoryResolver> {
    let mock_vcs: Arc<dyn VcsEntityRepository> =
        Arc::new(mock_services::MockVcsEntityRepository::new());
    Arc::new(RepositoryResolver::with_context(
        mock_vcs,
        ProjectContext::new("test/project", "project"),
    ))
}
