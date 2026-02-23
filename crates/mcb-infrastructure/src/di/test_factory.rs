//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! Test helpers for creating isolated domain services.
//!
//! # Architecture
//! This module provides factory functions to create `ServiceDependencies` with
//! isolated repositories (backed by a test-specific `DatabaseExecutor`) while
//! reusing heavy shared providers (embeddings, ONNX) from `AppContext`.
//!
//! This ensures that consumer crates (like `mcb-server`) can set up integration
//! tests without importing concrete infrastructure types (e.g., `Sqlite*`).

use std::sync::Arc;

use mcb_domain::ports::DatabaseExecutor;
use mcb_domain::registry::database::{
    DatabaseProviderConfig, list_database_providers, resolve_database_provider,
};

use crate::di::bootstrap::AppContext;
use crate::di::modules::domain_services::ServiceDependencies;

/// Create service dependencies with isolated repositories backed by the given executor.
///
/// This reuses shared providers (embeddings, vector store, cache) from `app_context`
/// but creates fresh SQLite-backed repositories using `executor`.
///
/// # Panics
///
/// Panics if:
/// - No database provider is registered (should never happen in tests)
/// - Database provider resolution fails (should never happen with built-in providers)
#[allow(clippy::panic)]
pub fn create_test_dependencies(
    project_id: String,
    executor: &Arc<dyn DatabaseExecutor>,
    app_context: &AppContext,
) -> ServiceDependencies {
    #[allow(clippy::map_unwrap_or)]
    let provider_name = list_database_providers()
        .first()
        .map(|(name, _)| *name)
        .unwrap_or_else(|| panic!("database provider must be registered for tests"));
    let provider = resolve_database_provider(&DatabaseProviderConfig::new(provider_name))
        .unwrap_or_else(|_| panic!("database provider resolution must succeed for tests"));

    // Fresh repositories backed by the isolated database
    let memory_repository = provider.create_memory_repository(Arc::clone(executor));
    let agent_repository = provider.create_agent_repository(Arc::clone(executor));
    let project_repository = provider.create_project_repository(Arc::clone(executor));
    let file_hash_repository =
        provider.create_file_hash_repository(Arc::clone(executor), project_id.clone());
    let vcs_entity_repository = provider.create_vcs_entity_repository(Arc::clone(executor));
    let plan_entity_repository = provider.create_plan_entity_repository(Arc::clone(executor));
    let issue_entity_repository = provider.create_issue_entity_repository(Arc::clone(executor));
    let org_entity_repository = provider.create_org_entity_repository(Arc::clone(executor));

    ServiceDependencies {
        project_id,
        cache: crate::cache::provider::SharedCacheProvider::from_arc(
            app_context.cache_handle().get(),
        ),
        crypto: app_context.crypto_service(),
        config: (*app_context.config).clone(),
        embedding_provider: app_context.embedding_handle().get(),
        vector_store_provider: app_context.vector_store_handle().get(),
        language_chunker: app_context.language_handle().get(),
        indexing_ops: app_context.indexing(),
        event_bus: app_context.event_bus(),
        file_system_provider: app_context.file_system_provider(),
        task_runner_provider: app_context.task_runner_provider(),
        memory_repository,
        agent_repository,
        file_hash_repository,
        vcs_provider: app_context.vcs_provider(),
        project_service: app_context.project_service(),
        project_repository,
        vcs_entity_repository,
        plan_entity_repository,
        issue_entity_repository,
        org_entity_repository,
    }
}
