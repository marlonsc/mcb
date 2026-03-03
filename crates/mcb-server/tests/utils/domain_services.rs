//! Domain-services test helpers (mcb-server specific).
//!
//! Builds [`McbState`] from pure registry DI — no infrastructure imports.
//! All providers resolved through `mcb_domain::registry::*` linkme slices.

use std::sync::Arc;

use mcb_domain::registry::ServiceResolutionContext;
use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
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

// linkme force-link only — DO NOT use for type/function imports (CA019 enforced)
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

/// Shared `FastEmbed` ONNX model cache directory (process-wide).
fn shared_fastembed_cache_dir() -> std::path::PathBuf {
    let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR")
        .or_else(|| std::env::var_os("FASTEMBED_CACHE_DIR"))
        .map_or_else(
            || std::env::temp_dir().join("mcb-fastembed-test-cache"),
            std::path::PathBuf::from,
        );
    let _ = std::fs::create_dir_all(&cache_dir);
    cache_dir
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

    // 3. Embedding — resolved through linkme registry
    let cache_dir = shared_fastembed_cache_dir();
    let embedding_config = EmbeddingProviderConfig::new("fastembed")
        .with_cache_dir(cache_dir)
        .with_dimensions(384);
    let embedding_provider = match resolve_embedding_provider(&embedding_config) {
        Ok(p) => p,
        Err(e) => {
            mcb_domain::warn!(
                "domain_services",
                "SKIPPED: Embedding provider unavailable (skipping test)",
                &e
            );
            return None;
        }
    };

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
    let resolution_ctx = ServiceResolutionContext {
        db: Arc::clone(&db),
        config: Arc::new(
            *mcb_domain::registry::config::resolve_config_provider(
                &mcb_domain::registry::config::ConfigProviderConfig::new("loco_yaml"),
            )
            .ok()?
            .load_config()
            .ok()?
            .downcast::<mcb_infrastructure::config::app::AppConfig>()
            .ok()?,
        ), // Real AppConfig loaded via CA/DI (ConfigProvider → load_config() → downcast)
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
