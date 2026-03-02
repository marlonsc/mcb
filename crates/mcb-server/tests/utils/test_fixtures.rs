//! Test fixtures for mcb-server tests.
//!
//! Provides factory functions for creating test data and temporary directories.
//! MCP server and state are built via [`mcb_server::build_mcp_server_bootstrap`]
//! with pure registry DI through `mcb_domain::registry::*`.
//!
//! **Entity fixtures and constants are centralized in `mcb_domain::test_utils`.**
//! This module re-exports them and adds mcb-server-specific helpers.

use std::sync::Arc;

use mcb_domain::ports::{EmbeddingProvider, VectorStoreProvider};
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
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::mcp_server::McpServer;
use mcb_server::state::McbState;
use mcb_server::tools::ExecutionFlow;
use tempfile::TempDir;

// Force linkme registration of all concrete providers
extern crate mcb_providers;

// -----------------------------------------------------------------------------
// Re-export ALL centralized constants and fixtures from mcb-domain SSOT
// -----------------------------------------------------------------------------
pub use mcb_domain::test_utils::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, TEST_EMBEDDING_DIMENSIONS, TEST_ORG_ID,
    TEST_ORG_ID_A, TEST_ORG_ID_B, TEST_PROJECT_ID, TEST_REPO_NAME, TEST_SESSION_ID,
    create_temp_codebase, create_test_admin_user, create_test_api_key, create_test_indexing_result,
    create_test_organization, create_test_team, create_test_team_member, create_test_user_with,
};

// Re-export domain_services helpers for integration tests
pub use super::domain_services::create_base_memory_args;

// Backward-compat aliases used by e2e tests (delegate to centralized SSOT)

/// Alias for [`create_test_organization`].
#[must_use]
pub fn test_organization(id: &str) -> mcb_domain::entities::Organization {
    create_test_organization(id)
}

/// Alias for [`create_test_user_with`].
#[must_use]
pub fn test_user(org_id: &str, email: &str) -> mcb_domain::entities::User {
    create_test_user_with(org_id, email)
}

/// Alias for [`create_test_admin_user`].
#[must_use]
pub fn test_admin_user(org_id: &str, email: &str) -> mcb_domain::entities::User {
    create_test_admin_user(org_id, email)
}

/// Alias for [`create_test_team`].
#[must_use]
pub fn test_team(org_id: &str, name: &str) -> mcb_domain::entities::Team {
    create_test_team(org_id, name)
}

/// Alias for [`create_test_team_member`].
#[must_use]
pub fn test_team_member(team_id: &str, user_id: &str) -> mcb_domain::entities::TeamMember {
    create_test_team_member(team_id, user_id)
}

/// Alias for [`create_test_api_key`].
#[must_use]
pub fn test_api_key(user_id: &str, org_id: &str, name: &str) -> mcb_domain::entities::ApiKey {
    create_test_api_key(user_id, org_id, name)
}

// -----------------------------------------------------------------------------
// Golden test helpers (shared by tests/golden and integration)
// -----------------------------------------------------------------------------

pub use mcb_domain::test_fixtures::{
    golden_content_to_string, golden_count_result_entries, golden_parse_results_found,
    sample_codebase_path,
};

// ---------------------------------------------------------------------------
// Shared FastEmbed cache directory
// ---------------------------------------------------------------------------

/// Process-wide shared `FastEmbed` ONNX model cache directory.
#[must_use]
pub fn shared_fastembed_test_cache_dir() -> std::path::PathBuf {
    static DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let cache_dir = std::env::var_os("FASTEMBED_CACHE_DIR")
            .or_else(|| std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR"))
            .map_or_else(
                || std::env::temp_dir().join("mcb-fastembed-test-cache"),
                std::path::PathBuf::from,
            );
        if let Err(err) = std::fs::create_dir_all(&cache_dir) {
            mcb_domain::warn!(
                "test_fixtures",
                "failed to create shared fastembed cache dir",
                &err.to_string()
            );
            return std::env::temp_dir().join("mcb-fastembed-test-cache");
        }
        cache_dir
    })
    .clone()
}

// ---------------------------------------------------------------------------
// SharedTestContext — DI-resolved providers for integration tests
// ---------------------------------------------------------------------------

/// Shared test context with DI-resolved providers.
///
/// Resolves embedding and vector store providers through the same linkme registry
/// as production code. Process-wide shared to avoid re-loading ONNX models per test.
pub struct SharedTestContext {
    /// DI-resolved embedding provider.
    embedding: Arc<dyn EmbeddingProvider>,
    /// DI-resolved vector store provider.
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl SharedTestContext {
    /// Get the DI-resolved embedding provider.
    #[must_use]
    pub fn embedding_provider(&self) -> Arc<dyn EmbeddingProvider> {
        Arc::clone(&self.embedding)
    }

    /// Get the DI-resolved vector store provider.
    #[must_use]
    pub fn vector_store_provider(&self) -> Arc<dyn VectorStoreProvider> {
        Arc::clone(&self.vector_store)
    }
}

fn create_shared_test_context() -> Option<SharedTestContext> {
    // Create a persistent multi-thread runtime for provider actor tasks.
    // FastEmbed and EdgeVec use `tokio::spawn` for actor loops that must outlive
    // individual `#[tokio::test]` runtimes (each test creates/drops its own runtime).
    // Intentionally leaked because SharedTestContext is process-wide via OnceLock.
    let persistent_rt = Box::leak(Box::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .ok()?,
    ));
    // Enter the persistent runtime so tokio::spawn calls (inside provider constructors)
    // target it instead of the calling test's short-lived runtime.
    let _guard = persistent_rt.enter();

    let cache_dir = shared_fastembed_test_cache_dir();

    // Resolve providers through domain registry (pure CA/DI/Linkme)
    let embedding_config = EmbeddingProviderConfig::new("fastembed")
        .with_cache_dir(cache_dir)
        .with_dimensions(384);
    let embedding = resolve_embedding_provider(&embedding_config).ok()?;

    let vs_config = VectorStoreProviderConfig::new("edgevec")
        .with_dimensions(384)
        .with_collection("default");
    let vector_store = resolve_vector_store_provider(&vs_config).ok()?;

    Some(SharedTestContext {
        embedding,
        vector_store,
    })
}

/// Process-wide shared test context. Builds once via linkme registry resolution.
#[must_use]
pub fn try_shared_app_context() -> Option<&'static SharedTestContext> {
    static CTX: std::sync::OnceLock<Option<SharedTestContext>> = std::sync::OnceLock::new();
    CTX.get_or_init(create_shared_test_context).as_ref()
}

/// Returns the shared test context, or an error if initialization failed.
///
/// # Errors
///
/// Returns an error if the shared context was not initialized.
pub fn shared_app_context() -> Result<&'static SharedTestContext, &'static str> {
    try_shared_app_context().ok_or("shared test context init failed")
}

// ---------------------------------------------------------------------------
// create_test_mcb_state — delegates to domain_services
// ---------------------------------------------------------------------------

/// Create an [`McbState`] with an isolated database via pure registry DI.
///
/// Thin wrapper around [`create_real_domain_services`](super::domain_services::create_real_domain_services).
/// Returns `Option` — `None` means a provider is unavailable and the test should skip.
pub async fn create_test_mcb_state() -> Option<(McbState, TempDir)> {
    super::domain_services::create_real_domain_services().await
}

// ---------------------------------------------------------------------------
// create_test_mcp_server
// ---------------------------------------------------------------------------

/// Create an MCP server with default providers (`SQLite`, `FastEmbed`, etc.) and an isolated DB.
///
/// Builds state via pure registry DI: resolve providers through `mcb_domain::registry::*`,
/// build [`ServiceResolutionContext`], then [`build_mcp_server_bootstrap`].
/// Each call gets its own [`TempDir`] and database.
///
/// Returns `(server, temp_dir)` — keep `temp_dir` alive for the test.
///
/// # Errors
///
/// Returns an error if the resolution context or MCP bootstrap could not be built.
pub async fn create_test_mcp_server() -> Result<(McpServer, TempDir), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");

    // Resolve all providers through domain registry
    let db_config = DatabaseProviderConfig::new("sqlite").with_path(db_path);
    let db = resolve_database_provider(&db_config).await?;

    let event_bus = resolve_event_bus_provider(&EventBusProviderConfig::new("inprocess"))?;

    let cache_dir = shared_fastembed_test_cache_dir();
    let embedding_config = EmbeddingProviderConfig::new("fastembed")
        .with_cache_dir(cache_dir)
        .with_dimensions(384);
    let embedding_provider = resolve_embedding_provider(&embedding_config)?;

    let vs_config = VectorStoreProviderConfig::new("edgevec")
        .with_dimensions(384)
        .with_collection("default");
    let vector_store_provider = resolve_vector_store_provider(&vs_config)?;

    let hybrid_search =
        resolve_hybrid_search_provider(&HybridSearchProviderConfig::new("default"))?;

    // Real AppConfig loaded via production serde_json path (YAML → JSON → AppConfig → validate)
    let app_config = mcb_infrastructure::config::load_app_config()?;

    let resolution_ctx = ServiceResolutionContext {
        db: Arc::clone(&db),
        config: Arc::new(app_config),
        event_bus,
        embedding_provider: Arc::clone(&embedding_provider),
        vector_store_provider: Arc::clone(&vector_store_provider),
    };

    let bootstrap = build_mcp_server_bootstrap(
        &resolution_ctx,
        db,
        embedding_provider,
        vector_store_provider,
        hybrid_search,
        ExecutionFlow::ServerHybrid,
    )?;
    let server = Arc::unwrap_or_clone(bootstrap.mcp_server);

    Ok((server, temp_dir))
}

/// Process-wide shared [`McbState`] for unit tests. Builds once via
/// [`create_real_domain_services`](super::domain_services::create_real_domain_services).
#[must_use]
pub fn try_shared_mcb_state() -> Option<&'static McbState> {
    static STATE: std::sync::OnceLock<Option<(McbState, Box<TempDir>)>> =
        std::sync::OnceLock::new();
    STATE.get_or_init(|| {
        // Spawn a separate thread so the new runtime doesn't conflict with an
        // existing #[tokio::test] runtime on the calling thread.
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(async { super::domain_services::create_real_domain_services().await })
        })
        .join()
        .ok()?
        .map(|(s, t)| (s, Box::new(t)))
    });
    STATE.get().and_then(|o| o.as_ref()).map(|(s, _)| s as &_)
}

/// Returns the shared `McbState`, or an error if initialization failed.
///
/// # Errors
///
/// Returns an error if the shared state was not initialized.
pub fn shared_mcb_state() -> Result<&'static McbState, &'static str> {
    try_shared_mcb_state().ok_or("shared McbState init failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test so fixture helpers are not reported as dead code in the unit test target.
    #[test]
    fn test_fixture_helpers_used_in_unit_target() {
        let (_temp, path) = create_temp_codebase();
        assert!(path.join("lib.rs").exists());
        let r = create_test_indexing_result(2, 10, 0);
        assert_eq!(r.files_processed, 2);
        assert!(!TEST_SESSION_ID.is_empty());
        assert!(!TEST_REPO_NAME.is_empty());
        assert!(!TEST_ORG_ID.is_empty());
    }
}
