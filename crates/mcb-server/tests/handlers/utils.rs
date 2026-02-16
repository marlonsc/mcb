use std::sync::Arc;

use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};
use mcb_domain::value_objects::SessionId;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesContainer, DomainServicesFactory, ServiceDependencies,
};
use mcb_providers::database::{
    SqliteFileHashConfig, SqliteFileHashRepository, SqliteIssueEntityRepository,
    SqliteMemoryRepository, SqliteOrgEntityRepository, SqlitePlanEntityRepository,
    SqliteProjectRepository, SqliteVcsEntityRepository, create_agent_repository_from_executor,
};
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};

use crate::test_utils::test_fixtures::try_shared_app_context;

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

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create a fresh SQLite database for this test
    let db_provider = resolve_database_provider(&DatabaseProviderConfig::new("sqlite"))
        .expect("resolve sqlite provider");
    let db_executor = db_provider
        .connect(&db_path)
        .await
        .expect("connect fresh test database");

    let project_id = "test-project".to_owned();

    // Fresh repositories backed by the isolated database
    let memory_repository = Arc::new(SqliteMemoryRepository::new(Arc::clone(&db_executor)));
    let agent_repository = create_agent_repository_from_executor(Arc::clone(&db_executor));
    let project_repository: Arc<dyn ProjectRepository> =
        Arc::new(SqliteProjectRepository::new(Arc::clone(&db_executor)));
    let file_hash_repository = Arc::new(SqliteFileHashRepository::new(
        Arc::clone(&db_executor),
        SqliteFileHashConfig::default(),
        project_id.clone(),
    ));
    let vcs_entity_repository = Arc::new(SqliteVcsEntityRepository::new(Arc::clone(&db_executor)));
    let plan_entity_repository =
        Arc::new(SqlitePlanEntityRepository::new(Arc::clone(&db_executor)));
    let issue_entity_repository =
        Arc::new(SqliteIssueEntityRepository::new(Arc::clone(&db_executor)));
    let org_entity_repository = Arc::new(SqliteOrgEntityRepository::new(Arc::clone(&db_executor)));

    // Reuse shared providers (embedding, vector store, cache, language)
    let deps = ServiceDependencies {
        project_id,
        cache: mcb_infrastructure::cache::provider::SharedCacheProvider::from_arc(
            ctx.cache_handle().get(),
        ),
        crypto: ctx.crypto_service(),
        config: (*ctx.config).clone(),
        embedding_provider: ctx.embedding_handle().get(),
        vector_store_provider: ctx.vector_store_handle().get(),
        language_chunker: ctx.language_handle().get(),
        indexing_ops: ctx.indexing(),
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

    let services = DomainServicesFactory::create_services(deps)
        .await
        .expect("build domain services");
    Some((services, temp_dir))
}
