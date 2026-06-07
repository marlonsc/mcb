//! Domain-services test helpers (mcb-server specific).
//!
//! Builds [`McbState`] from pure registry DI — no infrastructure imports.
//! All providers resolved through `mcb_domain::registry::*` linkme slices.

use std::sync::Arc;

use mcb_domain::registry::ServiceResolutionContext;
use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};
use mcb_domain::registry::events::{EventBusProviderConfig, resolve_event_bus_provider};
use mcb_domain::registry::hybrid_search::{
    HybridSearchProviderConfig, resolve_hybrid_search_provider,
};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use mcb_domain::value_objects::SessionId;
use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::state::McbState;
use mcb_server::tools::ExecutionFlow;

// Force linkme registration of all concrete providers
extern crate mcb_providers;

/// Helper to create a base `MemoryArgs` with common defaults.
#[must_use]
pub fn create_base_memory_args(
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

/// Build [`McbState`] with an isolated database per test via pure registry DI.
///
/// Uses `mcb_domain::registry::*` to resolve all providers and
/// [`build_mcp_server_bootstrap`] for the full MCP server composition.
/// No `mcb_infrastructure` imports.
pub async fn create_real_domain_services() -> Option<(McbState, tempfile::TempDir)> {
    let temp_dir = tempfile::tempdir().ok()?;
    let db_path = temp_dir.path().join("test.db");

    // 1. Database — resolved through linkme registry
    let db_config = DatabaseProviderConfig::new("sqlite").with_path(db_path);
    let db = resolve_database_provider(&db_config).await.ok()?;

    // 2. Event bus — resolved through linkme registry
    let event_bus = resolve_event_bus_provider(&EventBusProviderConfig::new("inprocess")).ok()?;

    // 3. Embedding — deterministic local provider (no ONNX model download;
    //    contract/state tests assert MCP wiring, not embedding quality)
    let embedding_provider = super::test_fixtures::create_test_embedding_provider(384);

    // 4. Vector store — resolved through linkme registry
    let vs_config = VectorStoreProviderConfig::new("edgevec")
        .with_dimensions(384)
        .with_collection("default");
    let vector_store_provider = match resolve_vector_store_provider(&vs_config) {
        Ok(p) => p,
        Err(e) => {
            mcb_domain::warn!(
                "domain_services",
                "SKIPPED: Vector store provider unavailable (skipping test)",
                &e
            );
            return None;
        }
    };

    // 5. Hybrid search — resolved through linkme registry
    let hybrid_search =
        resolve_hybrid_search_provider(&HybridSearchProviderConfig::new("default")).ok()?;

    // 6. Build ServiceResolutionContext (domain-level opaque DI context)
    // Real AppConfig: the indexing service builder downcasts ctx.config to AppConfig.
    let (app_config, _config_temp) =
        mcb_infrastructure::config::test_builder::TestConfigBuilder::new()
            .ok()?
            .build()
            .ok()?;
    let resolution_ctx = ServiceResolutionContext {
        db: Arc::clone(&db),
        config: Arc::new(app_config),
        event_bus,
        embedding_provider: Arc::clone(&embedding_provider),
        vector_store_provider: Arc::clone(&vector_store_provider),
    };

    // 7. Compose MCP server via Loco-style bootstrap (6-arg pure DI)
    let bootstrap = build_mcp_server_bootstrap(
        &resolution_ctx,
        db,
        embedding_provider,
        vector_store_provider,
        hybrid_search,
        ExecutionFlow::ServerHybrid,
    )
    .ok()?;

    let state = bootstrap.into_mcb_state();
    Some((state, temp_dir))
}
