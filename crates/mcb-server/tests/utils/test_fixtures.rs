//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.
//!
//! Uses a process-wide shared `AppContext` to avoid re-loading the ONNX model
//! (~5-10s) per test.  Each call to [`create_test_mcp_server`] gets an isolated
//! `SQLite` database backed by its own `TempDir`.

use mcb_domain::ports::services::IndexingResult;
use mcb_domain::registry::database::{DatabaseProviderConfig, resolve_database_provider};
use mcb_infrastructure::di::modules::domain_services::DomainServicesFactory;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_server::McpServerBuilder;
use mcb_server::mcp_server::McpServer;
use tempfile::TempDir;

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
// Shared AppContext (process-wide, ONNX model loaded once)
// ---------------------------------------------------------------------------

mcb_infrastructure::define_shared_test_context!("mcb-fixtures-shared.db");

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
    let temp_dir_result = tempfile::tempdir();
    assert!(temp_dir_result.is_ok(), "create temp dir");
    let temp_dir = match temp_dir_result {
        Ok(value) => value,
        Err(_) => {
            return (
                McpServerBuilder::new()
                    .build()
                    .unwrap_or_else(|_| unreachable!()),
                TempDir::new().unwrap_or_else(|_| unreachable!()),
            );
        }
    };
    let db_path = temp_dir.path().join("test.db");

    let db_provider_result = resolve_database_provider(&DatabaseProviderConfig::new("sqlite"));
    assert!(db_provider_result.is_ok(), "resolve sqlite provider");
    let db_provider = match db_provider_result {
        Ok(value) => value,
        Err(_) => {
            return (
                McpServerBuilder::new()
                    .build()
                    .unwrap_or_else(|_| unreachable!()),
                temp_dir,
            );
        }
    };
    let db_executor_result = db_provider.connect(&db_path).await;
    assert!(db_executor_result.is_ok(), "connect fresh test database");
    let db_executor = match db_executor_result {
        Ok(value) => value,
        Err(_) => {
            return (
                McpServerBuilder::new()
                    .build()
                    .unwrap_or_else(|_| unreachable!()),
                temp_dir,
            );
        }
    };

    let project_id = TEST_PROJECT_ID.to_owned();

    let deps = mcb_infrastructure::di::test_factory::create_test_dependencies(
        project_id,
        Arc::clone(&db_executor),
        &ctx,
    );

    let services_result = DomainServicesFactory::create_services(deps).await;
    assert!(services_result.is_ok(), "build domain services");
    let services = match services_result {
        Ok(value) => value,
        Err(_) => {
            return (
                McpServerBuilder::new()
                    .build()
                    .unwrap_or_else(|_| unreachable!()),
                temp_dir,
            );
        }
    };

    let server_result = McpServerBuilder::new()
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
        .build();
    assert!(server_result.is_ok(), "Failed to build MCP server");
    let server = match server_result {
        Ok(value) => value,
        Err(_) => {
            return (
                McpServerBuilder::new()
                    .build()
                    .unwrap_or_else(|_| unreachable!()),
                temp_dir,
            );
        }
    };

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
        assert!(!TEST_SESSION_ID.is_empty());
        assert!(!TEST_REPO_NAME.is_empty());
        assert!(!TEST_ORG_ID.is_empty());
    }
}
