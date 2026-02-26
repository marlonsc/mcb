use std::sync::Arc;

use mcb_domain::value_objects::SessionId;
use mcb_infrastructure::config::TestConfigBuilder;
use mcb_infrastructure::events::BroadcastEventBus;
use mcb_infrastructure::repositories::connect_sqlite_with_migrations;
use mcb_infrastructure::resolution_context::ServiceResolutionContext;
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::state::McbState;
use mcb_server::tools::ExecutionFlow;

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

/// Build [`McbState`] with an isolated database per test via Loco-style composition.
///
/// Uses [`ServiceResolutionContext`] and [`build_mcp_server_bootstrap`]; no manual bootstrap.
pub(crate) async fn create_real_domain_services() -> Option<(McbState, tempfile::TempDir)> {
    let builder = TestConfigBuilder::new().ok()?;
    let builder = builder.with_temp_db("test.db").ok()?;
    let builder = builder.with_fastembed_shared_cache().ok()?;
    let (config, opt_temp) = builder.build().ok()?;

    let temp_dir = opt_temp?;
    let db_path = config
        .providers
        .database
        .configs
        .get("default")
        .and_then(|c| c.path.as_ref())
        .cloned()
        .unwrap_or_else(|| temp_dir.path().join("test.db"));

    let db = connect_sqlite_with_migrations(&db_path).await.ok()?;

    let resolution_ctx = ServiceResolutionContext {
        db,
        config: Arc::new(config),
        event_bus: Arc::new(BroadcastEventBus::new()),
    };

    let bootstrap =
        build_mcp_server_bootstrap(&resolution_ctx, ExecutionFlow::ServerHybrid).ok()?;
    let state = bootstrap.into_mcb_state();
    Some((state, temp_dir))
}
