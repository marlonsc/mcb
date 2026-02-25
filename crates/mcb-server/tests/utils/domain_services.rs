use std::sync::Arc;

use mcb_domain::value_objects::SessionId;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesContainer, DomainServicesFactory,
};
use mcb_infrastructure::di::repositories::connect_sqlite_with_migrations;
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};

use crate::utils::test_fixtures::{TEST_PROJECT_ID, try_shared_app_context};

/// Helper to create a base `MemoryArgs` with common defaults
pub(crate) fn create_base_memory_args(
    action: MemoryAction,
    resource: MemoryResource,
    data: Option<serde_json::Value>,
    ids: Option<Vec<String>>,
    session_id: Option<String>,
) -> MemoryArgs {
    MemoryArgs {
        action,
        org_id: None,
        resource,
        project_id: None,
        data,
        ids,
        repo_id: None,
        session_id: session_id.map(|id| SessionId::from_string(&id)),
        parent_session_id: None,
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: None,
    }
}

/// Build domain services with an **isolated database** per test, reusing the
/// shared embedding/vector/cache/language providers from [`shared_app_context`].
pub(crate) async fn create_real_domain_services()
-> Option<(DomainServicesContainer, tempfile::TempDir)> {
    let ctx = try_shared_app_context()?;

    let temp_dir = tempfile::tempdir().ok()?;
    let db_path = temp_dir.path().join("test.db");

    // Create a fresh SQLite database via SeaORM
    let db = connect_sqlite_with_migrations(&db_path).await.ok()?;
    let db = Arc::new(db);

    let project_id = TEST_PROJECT_ID.to_owned();

    let repos = mcb_domain::registry::database::resolve_database_repositories(
        "seaorm",
        Box::new((*db).clone()),
        project_id.clone(),
    )
    .ok()?;
    let memory_repository = repos.memory;
    let agent_repository = repos.agent;
    let project_repository = repos.project;
    let vcs_entity_repository = repos.vcs_entity;
    let plan_entity_repository = repos.plan_entity;
    let issue_entity_repository = repos.issue_entity;
    let org_entity_repository = repos.org_entity;
    let file_hash_repository = repos.file_hash;

    let deps = mcb_infrastructure::di::modules::domain_services::ServiceDependencies {
        project_id,
        cache: mcb_infrastructure::cache::provider::SharedCacheProvider::from_arc(
            ctx.cache_provider(),
        ),
        crypto: ctx.crypto_service(),
        config: (*ctx.config).clone(),
        embedding_provider: ctx.embedding_provider(),
        vector_store_provider: ctx.vector_store_provider(),
        language_chunker: ctx.language_chunker(),
        indexing_ops: ctx.indexing(),
        validation_ops: ctx.validation_ops(),
        event_bus: ctx.event_bus(),
        memory_repository,
        agent_repository,
        file_hash_repository,
        vcs_provider: ctx.vcs_provider(),
        project_service: ctx.project_service(),
        project_repository,
        vcs_entity_repository,
        plan_entity_repository,
        issue_entity_repository,
        org_entity_repository,
    };

    let services = DomainServicesFactory::create_services(deps).await.ok()?;
    Some((services, temp_dir))
}
