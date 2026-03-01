//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.
//! MCP server and state are built via [`mcb_server::build_mcp_server_bootstrap`]
//! from a [`ServiceResolutionContext`] (Loco-style composition). No manual bootstrap.
//!
//! **Entity fixtures and constants are centralized in `mcb_domain::test_utils`.**
//! This module re-exports them and adds mcb-server-specific helpers.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_domain::ports::{EmbeddingProvider, VectorStoreProvider};
use mcb_domain::registry::events::{EventBusProviderConfig, resolve_event_bus_provider};
use mcb_infrastructure::config::TestConfigBuilder;
use mcb_infrastructure::repositories::connect_sqlite_with_migrations;
use mcb_infrastructure::resolution_context::{
    ServiceResolutionContext, resolve_embedding_from_config, resolve_vector_store_from_config,
};
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::mcp_server::McpServer;
use mcb_server::tools::ExecutionFlow;
use rstest::rstest;
use tempfile::TempDir;

// -----------------------------------------------------------------------------
// Re-export ALL centralized constants and fixtures from mcb-domain SSOT
// -----------------------------------------------------------------------------
pub use mcb_domain::test_utils::{
    GOLDEN_COLLECTION, SAMPLE_CODEBASE_FILES, TEST_EMBEDDING_DIMENSIONS, TEST_ORG_ID,
    TEST_ORG_ID_A, TEST_ORG_ID_B, TEST_PROJECT_ID, TEST_REPO_NAME, TEST_SESSION_ID,
    create_temp_codebase, create_test_admin_user, create_test_api_key, create_test_indexing_result,
    create_test_organization, create_test_team, create_test_team_member, create_test_user_with,
};

// Backward-compat aliases used by e2e tests (delegate to centralized SSOT)
/// Alias for [`create_test_organization`].
pub fn test_organization(id: &str) -> mcb_domain::entities::Organization {
    create_test_organization(id)
}
/// Alias for [`create_test_user_with`].
pub fn test_user(org_id: &str, email: &str) -> mcb_domain::entities::User {
    create_test_user_with(org_id, email)
}
/// Alias for [`create_test_admin_user`].
pub fn test_admin_user(org_id: &str, email: &str) -> mcb_domain::entities::User {
    create_test_admin_user(org_id, email)
}
/// Alias for [`create_test_team`].
pub fn test_team(org_id: &str, name: &str) -> mcb_domain::entities::Team {
    create_test_team(org_id, name)
}
/// Alias for [`create_test_team_member`].
pub fn test_team_member(team_id: &str, user_id: &str) -> mcb_domain::entities::TeamMember {
    create_test_team_member(team_id, user_id)
}
/// Alias for [`create_test_api_key`].
pub fn test_api_key(user_id: &str, org_id: &str, name: &str) -> mcb_domain::entities::ApiKey {
    create_test_api_key(user_id, org_id, name)
}

// -----------------------------------------------------------------------------
// Golden test helpers (shared by tests/golden and integration)
// -----------------------------------------------------------------------------

/// Path to `sample_codebase` fixture (used by golden tests).
pub fn sample_codebase_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_codebase")
}

/// Extract text content from `CallToolResult` for assertions (joined by space).
pub fn golden_content_to_string(res: &rmcp::model::CallToolResult) -> String {
    super::text::extract_text_with_sep(&res.content, " ")
}

/// Parse "**Results found:** N" from search response text.
pub fn golden_parse_results_found(text: &str) -> Option<usize> {
    let prefix = "**Results found:**";
    text.find(prefix).and_then(|i| {
        let rest = text[i + prefix.len()..].trim_start();
        let num_str: String = rest.chars().take_while(char::is_ascii_digit).collect();
        num_str.parse().ok()
    })
}

/// Count result lines (each has "📁") in search response.
pub fn golden_count_result_entries(text: &str) -> usize {
    text.lines().filter(|line| line.contains("📁")).count()
}

// SAMPLE_CODEBASE_FILES is now re-exported from mcb_domain::test_utils.

// NOTE: create_temp_codebase and create_test_indexing_result are now
// re-exported from mcb_domain::test_utils (see re-exports above).

// ---------------------------------------------------------------------------
// Shared AppContext (process-wide) with FastEmbed fallback
// ---------------------------------------------------------------------------

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
    embedding: Arc<dyn EmbeddingProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
}

impl SharedTestContext {
    /// Get the DI-resolved embedding provider.
    pub fn embedding_provider(&self) -> Arc<dyn EmbeddingProvider> {
        Arc::clone(&self.embedding)
    }

    /// Get the DI-resolved vector store provider.
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

    let builder = TestConfigBuilder::new().ok()?;
    let builder = builder.with_fastembed_shared_cache().ok()?;
    let (config, _opt_temp) = builder.build().ok()?;

    // Resolve providers through centralized helpers (same as production).
    let embedding = resolve_embedding_from_config(&config).ok()?;
    let vector_store = resolve_vector_store_from_config(&config).ok()?;

    Some(SharedTestContext {
        embedding,
        vector_store,
    })
}

/// Process-wide shared test context. Builds once via linkme registry resolution.
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

async fn create_test_resolution_context() -> Option<(ServiceResolutionContext, TempDir)> {
    let (config, opt_temp) = TestConfigBuilder::new()
        .ok()?
        .with_temp_db("test.db")
        .ok()?
        .with_fastembed_shared_cache()
        .ok()?
        .build()
        .ok()?;

    let temp_dir = opt_temp.unwrap_or_else(|| TempDir::new().unwrap_or_else(|_| unreachable!()));
    let db_path = config
        .providers
        .database
        .configs
        .get("default")
        .and_then(|c| c.path.as_ref())
        .cloned()
        .unwrap_or_else(|| temp_dir.path().join("test.db"));

    let db = connect_sqlite_with_migrations(&db_path).await.ok()?;
    let event_bus = resolve_event_bus_provider(&EventBusProviderConfig::new("inprocess")).ok()?;
    let embedding_provider = resolve_embedding_from_config(&config).ok()?;
    let vector_store_provider = resolve_vector_store_from_config(&config).ok()?;
    let resolution_ctx = ServiceResolutionContext {
        db,
        config: Arc::new(config),
        event_bus,
        embedding_provider,
        vector_store_provider,
    };
    Some((resolution_ctx, temp_dir))
}

// ---------------------------------------------------------------------------
// create_test_mcp_server
// ---------------------------------------------------------------------------

/// Create an MCP server with default providers (`SQLite`, `FastEmbed`, etc.) and an isolated DB.
///
/// Builds state via Loco-style composition: [`ServiceResolutionContext`] +
/// [`build_mcp_server_bootstrap`]. Each call gets its own [`TempDir`] and database.
///
/// Returns `(server, temp_dir)` — keep `temp_dir` alive for the test.
/// # Errors
///
/// Returns an error if the resolution context or MCP bootstrap could not be built.
pub async fn create_test_mcp_server() -> Result<(McpServer, TempDir), Box<dyn std::error::Error>> {
    let (resolution_ctx, temp_dir) = create_test_resolution_context()
        .await
        .ok_or("failed to build test ServiceResolutionContext")?;

    let bootstrap = build_mcp_server_bootstrap(&resolution_ctx, ExecutionFlow::ServerHybrid)?;
    let server = Arc::unwrap_or_clone(bootstrap.mcp_server);

    Ok((server, temp_dir))
}

/// Process-wide shared [`McbState`] for unit tests. Builds once via [`create_real_domain_services`].
pub fn try_shared_mcb_state() -> Option<&'static mcb_server::state::McbState> {
    static STATE: std::sync::OnceLock<
        Option<(mcb_server::state::McbState, Box<tempfile::TempDir>)>,
    > = std::sync::OnceLock::new();
    STATE.get_or_init(|| {
        // Spawn a separate thread so the new runtime doesn't conflict with an
        // existing #[tokio::test] runtime on the calling thread.
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(async {
                crate::utils::domain_services::create_real_domain_services().await
            })
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
pub fn shared_mcb_state() -> Result<&'static mcb_server::state::McbState, &'static str> {
    try_shared_mcb_state().ok_or("shared McbState init failed")
}

// NOTE: Entity fixture builders (test_organization, test_user, test_admin_user,
// test_team, test_team_member, test_api_key) are now thin wrappers that delegate
// to mcb_domain::test_utils. See the backward-compat aliases at the top of this file.

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Smoke test so fixture helpers are not reported as dead code in the unit test target.
    #[rstest]
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
