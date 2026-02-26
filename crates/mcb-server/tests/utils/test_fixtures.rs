//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.
//! MCP server and state are built via [`mcb_server::build_mcp_server_bootstrap`]
//! from a [`ServiceResolutionContext`] (Loco-style composition). No manual bootstrap.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_domain::entities::{
    ApiKey, Organization, Team, TeamMember, TeamMemberRole, User, UserRole,
};
use mcb_domain::ports::{EmbeddingProvider, IndexingResult, VectorStoreProvider};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use mcb_domain::utils::time::epoch_secs_i64;
use mcb_domain::value_objects::TeamMemberId;
use mcb_infrastructure::config::TestConfigBuilder;
use mcb_infrastructure::events::BroadcastEventBus;
use mcb_infrastructure::repositories::connect_sqlite_with_migrations;
use mcb_infrastructure::resolution_context::ServiceResolutionContext;
use mcb_server::build_mcp_server_bootstrap;
use mcb_server::mcp_server::McpServer;
use mcb_server::tools::ExecutionFlow;
use tempfile::TempDir;
use uuid::Uuid;

// -----------------------------------------------------------------------------
// Common test fixture constants
// -----------------------------------------------------------------------------

/// Test fixture: default project identifier.
pub const TEST_PROJECT_ID: &str = "test-project";

/// Test fixture: default session identifier.
pub const TEST_SESSION_ID: &str = "test-session";

/// Test fixture: default repository name.
pub const TEST_REPO_NAME: &str = "test-repo";

/// Test fixture: default organization identifier.
pub const TEST_ORG_ID: &str = "test-org";

/// Test fixture: default embedding dimensions (`FastEmbed` BGE-small-en-v1.5).
pub const TEST_EMBEDDING_DIMENSIONS: usize = 384;

/// Test fixture: organization A identifier for multi-tenant tests.
pub const TEST_ORG_ID_A: &str = "test-org-a";

/// Test fixture: organization B identifier for multi-tenant tests.
pub const TEST_ORG_ID_B: &str = "test-org-b";

// -----------------------------------------------------------------------------
// Golden test helpers (shared by tests/golden and integration)
// -----------------------------------------------------------------------------

pub const GOLDEN_COLLECTION: &str = "mcb_golden_test";

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

/// Expected files in `sample_codebase` for search assertions.
pub const SAMPLE_CODEBASE_FILES: &[&str] = &[
    "embedding.rs",
    "vector_store.rs",
    "handlers.rs",
    "cache.rs",
    "di.rs",
    "error.rs",
    "chunking.rs",
];

/// Create a temporary codebase directory with sample code files
pub fn create_temp_codebase() -> (TempDir, PathBuf) {
    let temp_dir_result = TempDir::new();
    assert!(temp_dir_result.is_ok(), "Failed to create temp directory");
    let temp_dir = match temp_dir_result {
        Ok(value) => value,
        Err(_) => {
            return (
                TempDir::new().unwrap_or_else(|_| unreachable!()),
                PathBuf::new(),
            );
        }
    };
    let codebase_path = temp_dir.path().to_path_buf();

    // Create sample Rust files
    let write_lib = std::fs::write(
        codebase_path.join("lib.rs"),
        r#"//! Sample library
pub fn hello() {
    println!("Hello, world!");
}
"#,
    );
    assert!(write_lib.is_ok(), "Failed to write lib.rs");

    let write_main = std::fs::write(
        codebase_path.join("main.rs"),
        "fn main() {
    mylib::hello();
}
",
    );
    assert!(write_main.is_ok(), "Failed to write main.rs");

    // Create a subdirectory with more files
    let src_dir = codebase_path.join("src");
    let mkdir_src = std::fs::create_dir_all(&src_dir);
    assert!(mkdir_src.is_ok(), "Failed to create src directory");

    let write_utils = std::fs::write(
        src_dir.join("utils.rs"),
        r#"pub fn helper() -> String {
    "helper".to_string()
}
"#,
    );
    assert!(write_utils.is_ok(), "Failed to write utils.rs");

    (temp_dir, codebase_path)
}

/// Create a test indexing result
pub fn create_test_indexing_result(
    files_processed: usize,
    chunks_created: usize,
    error_count: usize,
) -> IndexingResult {
    let errors = (0..error_count)
        .map(|i| format!("Test error {i}"))
        .collect();

    IndexingResult {
        files_processed,
        chunks_created,
        files_skipped: 0,
        errors,
        operation_id: None,
        status: "completed".to_owned(),
    }
}

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

    // Resolve embedding provider through linkme registry (same as production).
    // Propagate ALL config fields from AppConfig — single source of truth.
    let mut embed_cfg = EmbeddingProviderConfig::new(
        config
            .providers
            .embedding
            .provider
            .as_deref()
            .unwrap_or("fastembed"),
    );
    if let Some(ref v) = config.providers.embedding.cache_dir {
        embed_cfg = embed_cfg.with_cache_dir(v.clone());
    }
    if let Some(ref v) = config.providers.embedding.model {
        embed_cfg = embed_cfg.with_model(v.clone());
    }
    if let Some(ref v) = config.providers.embedding.base_url {
        embed_cfg = embed_cfg.with_base_url(v.clone());
    }
    if let Some(ref v) = config.providers.embedding.api_key {
        embed_cfg = embed_cfg.with_api_key(v.clone());
    }
    if let Some(d) = config.providers.embedding.dimensions {
        embed_cfg = embed_cfg.with_dimensions(d);
    }
    let embedding = resolve_embedding_provider(&embed_cfg).ok()?;

    // Resolve vector store provider through linkme registry (same as production).
    // Propagate ALL config fields from AppConfig — single source of truth.
    let mut vec_cfg = VectorStoreProviderConfig::new(
        config
            .providers
            .vector_store
            .provider
            .as_deref()
            .unwrap_or("edgevec"),
    );
    if let Some(ref v) = config.providers.vector_store.address {
        vec_cfg = vec_cfg.with_uri(v.clone());
    }
    if let Some(ref v) = config.providers.vector_store.collection {
        vec_cfg = vec_cfg.with_collection(v.clone());
    }
    if let Some(d) = config.providers.vector_store.dimensions {
        vec_cfg = vec_cfg.with_dimensions(d);
    }
    let vector_store = resolve_vector_store_provider(&vec_cfg).ok()?;

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

#[allow(clippy::panic, clippy::expect_used)]
pub fn shared_app_context() -> &'static SharedTestContext {
    try_shared_app_context().expect("shared test context init failed")
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
    let resolution_ctx = ServiceResolutionContext {
        db,
        config: Arc::new(config),
        event_bus: Arc::new(BroadcastEventBus::new()),
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
#[allow(clippy::panic)]
pub async fn create_test_mcp_server() -> (McpServer, TempDir) {
    let (resolution_ctx, temp_dir) = create_test_resolution_context()
        .await
        .unwrap_or_else(|| panic!("failed to build test ServiceResolutionContext"));

    let bootstrap = build_mcp_server_bootstrap(&resolution_ctx, ExecutionFlow::ServerHybrid)
        .unwrap_or_else(|err| panic!("failed to build MCP server bootstrap for tests: {err}"));
    let server = Arc::unwrap_or_clone(bootstrap.mcp_server);

    (server, temp_dir)
}

/// Process-wide shared [`McbState`] for unit tests. Builds once via [`create_real_domain_services`].
#[allow(clippy::unwrap_used)]
pub fn try_shared_mcb_state() -> Option<&'static mcb_server::state::McbState> {
    static STATE: std::sync::OnceLock<
        Option<(mcb_server::state::McbState, Box<tempfile::TempDir>)>,
    > = std::sync::OnceLock::new();
    STATE.get_or_init(|| {
        // Spawn a separate thread so the new runtime doesn't conflict with an
        // existing #[tokio::test] runtime on the calling thread.
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                crate::utils::domain_services::create_real_domain_services().await
            })
        })
        .join()
        .unwrap()
        .map(|(s, t)| (s, Box::new(t)))
    });
    STATE.get().and_then(|o| o.as_ref()).map(|(s, _)| s as &_)
}

#[allow(clippy::panic, clippy::expect_used)]
pub fn shared_mcb_state() -> &'static mcb_server::state::McbState {
    try_shared_mcb_state().expect("shared McbState init failed")
}

// -----------------------------------------------------------------------------
// Test Fixture Builders — Org/User/ApiKey/Team/TeamMember (used by e2e/contract/integration)
// -----------------------------------------------------------------------------

/// Create a test organization with sensible defaults.
pub fn test_organization(id: &str) -> Organization {
    Organization {
        id: id.to_owned(),
        name: format!("Test Org {id}"),
        slug: format!("test-org-{id}"),
        settings_json: "{}".to_owned(),
        created_at: epoch_secs_i64().unwrap_or(0),
        updated_at: epoch_secs_i64().unwrap_or(0),
    }
}

/// Create a test user with Member role.
pub fn test_user(org_id: &str, email: &str) -> User {
    User {
        id: Uuid::new_v4().to_string(),
        org_id: org_id.to_owned(),
        email: email.to_owned(),
        display_name: email.split('@').next().unwrap_or("Test User").to_owned(),
        role: UserRole::Member,
        api_key_hash: None,
        created_at: epoch_secs_i64().unwrap_or(0),
        updated_at: epoch_secs_i64().unwrap_or(0),
    }
}

/// Create a test user with Admin role.
pub fn test_admin_user(org_id: &str, email: &str) -> User {
    User {
        id: Uuid::new_v4().to_string(),
        org_id: org_id.to_owned(),
        email: email.to_owned(),
        display_name: email.split('@').next().unwrap_or("Test Admin").to_owned(),
        role: UserRole::Admin,
        api_key_hash: None,
        created_at: epoch_secs_i64().unwrap_or(0),
        updated_at: epoch_secs_i64().unwrap_or(0),
    }
}

/// Create a test team.
pub fn test_team(org_id: &str, name: &str) -> Team {
    Team {
        id: Uuid::new_v4().to_string(),
        org_id: org_id.to_owned(),
        name: name.to_owned(),
        created_at: epoch_secs_i64().unwrap_or(0),
    }
}

/// Create a test team member.
pub fn test_team_member(team_id: &str, user_id: &str) -> TeamMember {
    TeamMember {
        id: TeamMemberId::from_string(&format!("{team_id}:{user_id}")),
        team_id: team_id.to_owned(),
        user_id: user_id.to_owned(),
        role: TeamMemberRole::Member,
        joined_at: epoch_secs_i64().unwrap_or(0),
    }
}

/// Create a test API key.
pub fn test_api_key(user_id: &str, org_id: &str, name: &str) -> ApiKey {
    ApiKey {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_owned(),
        org_id: org_id.to_owned(),
        name: name.to_owned(),
        key_hash: format!("hash_{}", Uuid::new_v4()),
        scopes_json: "[\"read\", \"write\"]".to_owned(),
        expires_at: None,
        revoked_at: None,
        created_at: epoch_secs_i64().unwrap_or(0),
    }
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
