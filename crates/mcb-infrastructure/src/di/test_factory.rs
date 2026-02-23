#![allow(clippy::must_use_candidate, missing_docs)]

use std::sync::Arc;

use sea_orm::DatabaseConnection;

use mcb_providers::database::seaorm::repos::{
    SeaOrmAgentRepository, SeaOrmEntityRepository, SeaOrmIndexRepository,
    SeaOrmObservationRepository, SeaOrmProjectRepository,
};

use crate::di::bootstrap::AppContext;
use crate::di::modules::domain_services::ServiceDependencies;

pub fn create_test_dependencies(
    project_id: String,
    db: &Arc<DatabaseConnection>,
    app_context: &AppContext,
) -> ServiceDependencies {
    let memory_repository = Arc::new(SeaOrmObservationRepository::new((**db).clone()));
    let agent_repository = Arc::new(SeaOrmAgentRepository::new(Arc::clone(db)));
    let project_repository = Arc::new(SeaOrmProjectRepository::new((**db).clone()));
    let file_hash_repository = Arc::new(SeaOrmIndexRepository::new(
        Arc::clone(db),
        project_id.clone(),
    ));
    let entity_repo = Arc::new(SeaOrmEntityRepository::new(Arc::clone(db)));

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
        vcs_entity_repository: Arc::clone(&entity_repo) as _,
        plan_entity_repository: Arc::clone(&entity_repo) as _,
        issue_entity_repository: Arc::clone(&entity_repo) as _,
        org_entity_repository: entity_repo,
    }
}
