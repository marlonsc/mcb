//! Test fixtures for mcb-server tests
//!
//! Provides factory functions for creating test data and temporary directories.

#![allow(dead_code)]

use mcb_application::ValidationService;
use mcb_application::domain_services::search::{IndexingResult, IndexingStatus};
use mcb_domain::SearchResult;
use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::config::types::AppConfig;
use mcb_infrastructure::crypto::CryptoService;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_infrastructure::di::modules::domain_services::{
    DomainServicesFactory, ServiceDependencies,
};
use mcb_server::McpServerBuilder;
use mcb_server::mcp_server::McpServer;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

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

/// Create a test indexing result with specific error messages
pub fn create_test_indexing_result_with_errors(
    files_processed: usize,
    chunks_created: usize,
    errors: Vec<String>,
) -> IndexingResult {
    IndexingResult {
        files_processed,
        chunks_created,
        files_skipped: 0,
        errors,
        operation_id: None,
        status: "completed".to_string(),
    }
}

/// Create an idle indexing status (not indexing)
pub fn create_idle_status() -> IndexingStatus {
    IndexingStatus {
        is_indexing: false,
        progress: 0.0,
        current_file: None,
        total_files: 0,
        processed_files: 0,
    }
}

/// Create an in-progress indexing status
pub fn create_in_progress_status(progress: f64, current_file: &str) -> IndexingStatus {
    IndexingStatus {
        is_indexing: true,
        progress,
        current_file: Some(current_file.to_string()),
        total_files: 100,
        processed_files: (progress * 100.0) as usize,
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

    // Create crypto service with random key for tests
    let master_key = CryptoService::generate_master_key();
    let crypto = CryptoService::new(master_key).expect("Failed to create crypto service");

    // Create domain services
    let deps = ServiceDependencies {
        cache: shared_cache,
        crypto,
        config,
        embedding_provider,
        vector_store_provider,
        language_chunker,
        indexing_ops,
        event_bus,
    };

    let services = DomainServicesFactory::create_services(deps)
        .await
        .expect("Failed to create services");

    let validation_service = Arc::new(ValidationService::new());

    McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(validation_service)
        .build()
        .expect("Failed to build MCP server")
}
