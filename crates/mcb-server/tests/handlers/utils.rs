use std::sync::Arc;

use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};
use mcb_domain::value_objects::SessionId;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesContainer, DomainServicesFactory,
};
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

    // Create a fresh SQLite database for this test
    let db_provider = resolve_database_provider(&DatabaseProviderConfig::new("sqlite")).ok()?;
    let db_executor = db_provider.connect(&db_path).await.ok()?;

    let project_id = TEST_PROJECT_ID.to_owned();

    let deps = mcb_infrastructure::di::test_factory::create_test_dependencies(
        project_id,
        Arc::clone(&db_executor),
        &ctx,
    );

    let services = DomainServicesFactory::create_services(deps).await.ok()?;
    Some((services, temp_dir))
}
