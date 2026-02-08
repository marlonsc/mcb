//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_application::use_cases::project_service::ProjectServiceImpl;
use mcb_domain::SearchResult;
use mcb_domain::ports::services::IndexingResult;
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::config::types::AppConfig;
use mcb_infrastructure::crypto::CryptoService;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesFactory, ServiceDependencies,
};
use mcb_server::McpServerBuilder;
use mcb_server::mcp_server::McpServer;
use tempfile::TempDir;

#[allow(unused_imports)]
use crate::test_utils::mock_services::MockMemoryRepository;
use crate::test_utils::mock_services::MockVcsProvider;

// -----------------------------------------------------------------------------
// Golden test helpers (shared by tests/golden and integration)
// -----------------------------------------------------------------------------

pub const GOLDEN_COLLECTION: &str = "mcb_golden_test";

/// Path to sample_codebase fixture (used by golden tests).
pub fn sample_codebase_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_codebase")
}

/// Extract text content from CallToolResult for assertions (joined by space).
pub fn golden_content_to_string(res: &rmcp::model::CallToolResult) -> String {
    extract_text_content_with_sep(&res.content, " ")
}

/// Extract text content from Content slice, joining with newline.
///
/// Shared helper used by golden integration tests and tools e2e tests.
#[allow(dead_code)]
pub fn extract_text_content(content: &[rmcp::model::Content]) -> String {
    extract_text_content_with_sep(content, "\n")
}

/// Internal helper: extract text from Content with a configurable separator.
fn extract_text_content_with_sep(content: &[rmcp::model::Content], sep: &str) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(v) = serde_json::to_value(c) {
                v.get("text").and_then(|t| t.as_str()).map(String::from)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// Parse "**Results found:** N" from search response text.
pub fn golden_parse_results_found(text: &str) -> Option<usize> {
    let prefix = "**Results found:**";
    text.find(prefix).and_then(|i| {
        let rest = text[i + prefix.len()..].trim_start();
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse().ok()
    })
}

/// Count result lines (each has "📁") in search response.
pub fn golden_count_result_entries(text: &str) -> usize {
    text.lines().filter(|line| line.contains("📁")).count()
}

/// Expected files in sample_codebase for search assertions.
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
        r#"fn main() {
    mylib::hello();
}
"#,
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
        .map(|i| format!("Test error {}", i))
        .collect();

    IndexingResult {
        files_processed,
        chunks_created,
        files_skipped: 0,
        errors,
        operation_id: None,
        status: "completed".to_string(),
    }
}

/// Create a single test search result
pub fn create_test_search_result(
    file_path: &str,
    content: &str,
    score: f64,
    start_line: u32,
) -> SearchResult {
    SearchResult {
        id: format!("test-result-{}", start_line),
        file_path: file_path.to_string(),
        start_line,
        content: content.to_string(),
        score,
        language: "rust".to_string(),
    }
}

/// Create multiple test search results
pub fn create_test_search_results(count: usize) -> Vec<SearchResult> {
    (0..count)
        .map(|i| {
            create_test_search_result(
                &format!("src/file_{}.rs", i),
                &format!("fn test_function_{}() {{\n    // test code\n}}", i),
                0.95 - (i as f64 * 0.05),
                (i as u32) * 10 + 1,
            )
        })
        .collect()
}

/// Create an MCP server with null providers for testing
///
/// This uses the default AppConfig which initializes null providers,
/// suitable for unit tests that don't need real embedding/vector store.
pub async fn create_test_mcp_server() -> McpServer {
    let config = AppConfig::default();
    let ctx = init_app(config.clone()).await.expect("Failed to init app");

    // Get providers from context
    let embedding_provider = ctx.embedding_handle().get();
    let vector_store_provider = ctx.vector_store_handle().get();
    let language_chunker = ctx.language_handle().get();
    let cache_provider = ctx.cache_handle().get();
    let indexing_ops = ctx.indexing();
    let event_bus = ctx.event_bus();

    // Create shared cache provider for domain services factory
    let shared_cache = SharedCacheProvider::from_arc(cache_provider);

    let master_key = CryptoService::generate_master_key();
    let crypto: std::sync::Arc<dyn mcb_domain::ports::providers::CryptoProvider> =
        std::sync::Arc::new(
            CryptoService::new(master_key).expect("Failed to create crypto service"),
        );

    let (memory_repository, shared_executor) =
        mcb_providers::database::create_memory_repository_in_memory_with_executor()
            .await
            .expect("Failed to create memory database");
    let agent_repository = mcb_providers::database::create_agent_repository_from_executor(
        std::sync::Arc::clone(&shared_executor),
    );
    let project_repository = mcb_providers::database::create_project_repository_from_executor(
        std::sync::Arc::clone(&shared_executor),
    );
    let vcs_provider = Arc::new(MockVcsProvider::new());

    let project_service: Arc<dyn mcb_domain::ports::services::ProjectDetectorService> =
        Arc::new(mcb_infrastructure::project::ProjectService::new());
    let project_workflow_service: Arc<dyn mcb_domain::ports::services::ProjectService> =
        Arc::new(ProjectServiceImpl::new(project_repository.clone()));

    let deps = ServiceDependencies {
        project_id: "test-project".to_string(),
        cache: shared_cache,
        crypto,
        config,
        embedding_provider,
        vector_store_provider,
        language_chunker,
        indexing_ops,
        event_bus,
        memory_repository,
        agent_repository,
        vcs_provider,
        project_service,
        project_workflow_service: project_workflow_service.clone(),
    };

    let services = DomainServicesFactory::create_services(deps)
        .await
        .expect("Failed to create services");

    McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .with_project_service(services.project_service)
        .with_project_workflow_service(project_workflow_service)
        .with_vcs_provider(services.vcs_provider)
        .build()
        .expect("Failed to build MCP server")
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
