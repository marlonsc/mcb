//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.
//!
//! Uses a process-wide shared `AppContext` to avoid re-loading the ONNX model
//! (~5-10s) per test.  Each call to [`create_test_mcp_server`] gets an isolated
//! `SQLite` database backed by its own `TempDir`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use mcb_domain::ports::repositories::ProjectRepository;
use mcb_domain::ports::services::IndexingResult;
use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesFactory, ServiceDependencies,
};
use mcb_providers::database::{
    SqliteFileHashConfig, SqliteFileHashRepository, SqliteIssueEntityRepository,
    SqliteMemoryRepository, SqliteOrgEntityRepository, SqlitePlanEntityRepository,
    SqliteProjectRepository, SqliteVcsEntityRepository, create_agent_repository_from_executor,
};
use mcb_server::McpServerBuilder;
use mcb_server::mcp_server::McpServer;
use tempfile::TempDir;

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
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let codebase_path = temp_dir.path().to_path_buf();

    // Create sample Rust files
    std::fs::write(
        codebase_path.join("lib.rs"),
        r#"//! Sample library
pub fn hello() {
    println!("Hello, world!");
}
"#,
    )
    .expect("Failed to write lib.rs");

    std::fs::write(
        codebase_path.join("main.rs"),
        "fn main() {
    mylib::hello();
}
",
    )
    .expect("Failed to write main.rs");

    // Create a subdirectory with more files
    let src_dir = codebase_path.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    std::fs::write(
        src_dir.join("utils.rs"),
        r#"pub fn helper() -> String {
    "helper".to_string()
}
"#,
    )
    .expect("Failed to write utils.rs");

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
// Shared AppContext (process-wide, ONNX model loaded once)
// ---------------------------------------------------------------------------

/// Process-wide shared `AppContext`.
///
/// Loads the ONNX embedding model exactly once and reuses it across all tests.
pub fn try_shared_app_context() -> Option<&'static AppContext> {
    static CTX: OnceLock<Option<AppContext>> = OnceLock::new();

    CTX.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("create init runtime");
            let result = rt.block_on(async {
                let temp_dir = tempfile::tempdir().expect("create temp dir");
                let temp_root = temp_dir.keep();
                let temp_path = temp_root.join("mcb-fixtures-shared.db");

                let mut config = ConfigLoader::new().load().expect("load config");
                config.providers.database.configs.insert(
                    "default".to_owned(),
                    mcb_infrastructure::config::DatabaseConfig {
                        provider: "sqlite".to_owned(),
                        path: Some(temp_path),
                    },
                );
                config.providers.embedding.cache_dir = Some(shared_fastembed_test_cache_dir());
                init_app(config).await
            });
            match result {
                Ok(ctx) => Some(ctx),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("model.onnx") || msg.contains("Failed to initialize FastEmbed") {
                        eprintln!(
                            "Skipping tests requiring shared AppContext: embedding model unavailable in offline env: {e}"
                        );
                        None
                    } else {
                        panic!("shared init_app failed: {e}");
                    }
                }
            }
        })
        .join()
        .expect("init thread panicked")
    })
    .as_ref()
}

pub fn shared_app_context() -> &'static AppContext {
    try_shared_app_context().expect("shared AppContext init failed - ONNX model may be unavailable")
}

/// Persistent shared cache dir for `FastEmbed` ONNX model.
///
/// Avoids re-downloading the model on every test invocation by using a
/// process-wide directory outside the per-test temp dirs.
pub fn shared_fastembed_test_cache_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR").map_or_else(
            || std::env::temp_dir().join("mcb-fastembed-test-cache"),
            PathBuf::from,
        );
        std::fs::create_dir_all(&cache_dir).expect("create shared fastembed test cache dir");
        cache_dir
    })
    .clone()
}

// ---------------------------------------------------------------------------
// create_test_mcp_server
// ---------------------------------------------------------------------------

/// Create an MCP server with default providers (`SQLite`, `EdgeVec`, `FastEmbed`, Tokio)
///
/// Reuses the process-wide [`shared_app_context`] so the ONNX embedding model
/// is loaded only once, but gives each call an **isolated `SQLite` database**
/// backed by its own `TempDir`.
///
/// Returns `(server, temp_dir)` -- `temp_dir` must be kept alive by the caller.
pub async fn create_test_mcp_server() -> (McpServer, TempDir) {
    let ctx = shared_app_context();

    // Fresh temp dir and database for this test
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

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

    let server = McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .with_project_service(services.project_service)
        .with_project_workflow_service(services.project_repository)
        .with_vcs_provider(services.vcs_provider)
        .with_vcs_entity_repository(services.vcs_entity_repository)
        .with_plan_entity_repository(services.plan_entity_repository)
        .with_issue_entity_repository(services.issue_entity_repository)
        .with_org_entity_repository(services.org_entity_repository)
        .build()
        .expect("Failed to build MCP server");

    (server, temp_dir)
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
    }
}
