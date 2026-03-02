//! Error shape tests for MCP handler error responses.
//!
//! All test infrastructure sourced from `mcb_domain::utils::tests` (SSOT).
//! Server-specific helpers (`MemoryArgs` factory, `McbState` bootstrap) are defined
//! locally since they depend on `mcb_server` types.

// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;

use rstest::{fixture, rstest};

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
use mcb_domain::utils::tests::mcp_assertions::assert_error_shape;
use mcb_domain::value_objects::SessionId;

use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource, SessionAction, SessionArgs};
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::handlers::{MemoryHandler, SessionHandler};
use mcb_server::state::McbState;
use mcb_server::tools::ExecutionFlow;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Local helpers (depend on mcb_server types — cannot live in mcb_domain)
// ---------------------------------------------------------------------------

/// Helper to create a base `MemoryArgs` with common defaults.
fn create_base_memory_args(
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
/// All providers resolved through `mcb_domain::registry` (CA/DI/Linkme).
async fn create_test_mcb_state() -> Option<(McbState, tempfile::TempDir)> {
    let temp_dir = tempfile::TempDir::new().ok()?;
    let db_path = temp_dir.path().join("test.db");

    let db_config = DatabaseProviderConfig::new("sqlite").with_path(db_path);
    let db = resolve_database_provider(&db_config).await.ok()?;

    let embed_config = EmbeddingProviderConfig::new("fastembed");
    let embedding_provider = resolve_embedding_provider(&embed_config).ok()?;

    let vec_config = VectorStoreProviderConfig::new("edgevec")
        .with_dimensions(384)
        .with_collection("default");
    let vector_store_provider = resolve_vector_store_provider(&vec_config).ok()?;

    let event_bus = resolve_event_bus_provider(&EventBusProviderConfig::new("inprocess")).ok()?;
    let hybrid_search =
        resolve_hybrid_search_provider(&HybridSearchProviderConfig::new("default")).ok()?;

    // Real AppConfig loaded via production serde_json path
    let app_config = mcb_infrastructure::config::load_app_config().ok()?;
    let resolution_ctx = ServiceResolutionContext {
        db,
        config: Arc::new(app_config),
        event_bus,
        embedding_provider,
        vector_store_provider,
    };

    let bootstrap = build_mcp_server_bootstrap(
        &resolution_ctx,
        Arc::clone(&resolution_ctx.db),
        Arc::clone(&resolution_ctx.embedding_provider),
        Arc::clone(&resolution_ctx.vector_store_provider),
        hybrid_search,
        ExecutionFlow::ServerHybrid,
    )
    .ok()?;
    let state = bootstrap.into_mcb_state();
    Some((state, temp_dir))
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

async fn memory_handler() -> Option<(MemoryHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_test_mcb_state().await?;
    Some((
        MemoryHandler::new(state.mcp_server.memory_service()),
        temp_dir,
    ))
}

async fn session_handler() -> Option<(SessionHandler, tempfile::TempDir)> {
    let (state, temp_dir) = create_test_mcb_state().await?;
    Some((
        SessionHandler::new(
            state.mcp_server.agent_session_service(),
            state.mcp_server.memory_service(),
        ),
        temp_dir,
    ))
}

#[fixture]
fn observation_store_args() -> MemoryArgs {
    create_base_memory_args(
        MemoryAction::Store,
        MemoryResource::Observation,
        None,
        None,
        Some("session-1".to_owned()),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
async fn memory_store_missing_data_returns_expected_error(
    #[from(observation_store_args)] args: MemoryArgs,
) {
    let Some((handler, _temp_dir)) = memory_handler().await else {
        return;
    };
    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");

    assert_error_shape(&response, "Missing data payload for observation store");
}

#[rstest]
#[tokio::test]
async fn memory_store_missing_content_returns_expected_error(
    #[from(observation_store_args)] mut args: MemoryArgs,
) {
    args.data = Some(json!({
        "observation_type": "code",
        "project_id": "project-1"
    }));

    let Some((handler, _temp_dir)) = memory_handler().await else {
        return;
    };
    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");

    assert_error_shape(&response, "Missing required field: content");
}

#[rstest]
#[tokio::test]
async fn session_create_missing_data_returns_expected_error() {
    let Some((handler, _temp_dir)) = session_handler().await else {
        return;
    };
    let args = SessionArgs {
        action: SessionAction::Create,
        org_id: None,
        session_id: None,
        data: None,
        project_id: None,
        worktree_id: None,
        parent_session_id: None,
        agent_type: Some("explore".to_owned()),
        status: None,
        limit: None,
    };

    let response = handler
        .handle(Parameters(args))
        .await
        .expect("handler response");
    assert_error_shape(&response, "Missing data payload for create");
}

#[rstest]
#[case(json!({"action": "bad_action", "resource": "observation"}), "action")]
#[case(json!({"action": "store", "resource": "bad_resource"}), "resource")]
fn invalid_enum_payloads_fail_deserialization(#[case] payload: Value, #[case] expected: &str) {
    let err = serde_json::from_value::<MemoryArgs>(payload).expect_err("invalid enum should fail");
    let message = err.to_string();

    assert!(
        message.contains(expected),
        "expected '{expected}' in deserialization error: {message}"
    );
    assert!(
        message.contains("unknown variant") || message.contains("invalid value"),
        "error should describe enum variant mismatch: {message}"
    );
}
