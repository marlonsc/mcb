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

use mcb_providers::database::{
    SqliteAgentRepository, SqliteFileHashConfig, SqliteFileHashRepository,
    SqliteIssueEntityRepository, SqliteMemoryRepository, SqliteOrgEntityRepository,
    SqlitePlanEntityRepository, SqliteProjectRepository, SqliteVcsEntityRepository,
};

use crate::di::bootstrap::AppContext;
use crate::di::modules::domain_services::ServiceDependencies;

/// Create service dependencies with isolated repositories backed by the given executor.
///
/// This reuses shared providers (embeddings, vector store, cache) from `app_context`
/// but creates fresh SQLite-backed repositories using `executor`.
pub fn create_test_dependencies(
    project_id: String,
    executor: &Arc<dyn DatabaseExecutor>,
    app_context: &AppContext,
) -> ServiceDependencies {
    // Fresh repositories backed by the isolated database
    let memory_repository = Arc::new(SqliteMemoryRepository::new(Arc::clone(executor)));
    let agent_repository = Arc::new(SqliteAgentRepository::new(Arc::clone(executor)));
    let project_repository = Arc::new(SqliteProjectRepository::new(Arc::clone(executor)));

    let file_hash_repository = Arc::new(SqliteFileHashRepository::new(
        Arc::clone(executor),
        SqliteFileHashConfig::default(),
        project_id.clone(),
    ));

    let vcs_entity_repository = Arc::new(SqliteVcsEntityRepository::new(Arc::clone(executor)));
    let plan_entity_repository = Arc::new(SqlitePlanEntityRepository::new(Arc::clone(executor)));
    let issue_entity_repository = Arc::new(SqliteIssueEntityRepository::new(Arc::clone(executor)));
    let org_entity_repository = Arc::new(SqliteOrgEntityRepository::new(Arc::clone(executor)));

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
