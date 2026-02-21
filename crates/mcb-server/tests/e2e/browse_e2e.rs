//! E2E browse tests with real `EdgeVec` vector store
//!
//! Tests the full browse API stack with a real in-memory vector store,
//! validating collection listing, file browsing, and chunk retrieval.
//!
//! Uses infrastructure factories — no direct `mcb_providers` imports.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use mcb_domain::ports::HighlightServiceInterface;
use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::ports::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_domain::value_objects::{CollectionId, Embedding};
use mcb_infrastructure::infrastructure::{
    AtomicPerformanceMetrics, DefaultIndexingOperations, create_test_vector_store_for_e2e,
    default_event_bus,
};
use mcb_infrastructure::services::highlight_service::HighlightServiceImpl;

use mcb_server::admin::auth::AdminAuthConfig;
use mcb_server::admin::browse_handlers::BrowseState;
use mcb_server::admin::handlers::AdminState;
use mcb_server::transport::axum_http::{AppState, build_router};
use tower::ServiceExt;

use crate::utils::test_fixtures::TEST_EMBEDDING_DIMENSIONS;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_highlight_service() -> Arc<dyn HighlightServiceInterface> {
    Arc::new(HighlightServiceImpl::new())
}

/// Create test admin state with minimal dependencies
fn create_test_admin_state() -> Result<AdminState, Box<dyn std::error::Error>> {
    Ok(AdminState {
        metrics: AtomicPerformanceMetrics::new_shared(),
        indexing: DefaultIndexingOperations::new_shared(),
        config_watcher: None,
        current_config: mcb_infrastructure::config::ConfigLoader::new().load()?,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: default_event_bus(),
        service_manager: None,
        cache: None,
        project_workflow: None,
        vcs_entity: None,
        plan_entity: None,
        issue_entity: None,
        org_entity: None,
        tool_handlers: None,
    })
}

#[derive(Clone)]
struct TestClient {
    app: Router,
}

struct Header {
    name: String,
    value: String,
}

impl Header {
    fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

struct TestResponse {
    status: StatusCode,
    body: String,
}

impl TestResponse {
    fn status(&self) -> StatusCode {
        self.status
    }

    async fn into_string(self) -> Option<String> {
        Some(self.body)
    }
}

struct RequestBuilder {
    app: Router,
    method: Method,
    path: String,
    headers: Vec<(String, String)>,
}

impl RequestBuilder {
    fn header(mut self, header: Header) -> Self {
        self.headers.push((header.name, header.value));
        self
    }

    async fn dispatch(self) -> TestResponse {
        let mut builder = Request::builder().method(self.method).uri(self.path);
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }
        let req = builder.body(Body::empty()).expect("valid request");
        let resp = self
            .app
            .oneshot(req)
            .await
            .expect("router should handle request");
        let status = resp.status();
        let body = resp
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        TestResponse {
            status,
            body: String::from_utf8(body.to_vec()).unwrap_or_default(),
        }
    }
}

impl TestClient {
    fn get(&self, path: &str) -> RequestBuilder {
        RequestBuilder {
            app: self.app.clone(),
            method: Method::GET,
            path: path.to_owned(),
            headers: Vec::new(),
        }
    }
}

async fn create_test_client(
    browse_state: BrowseState,
) -> Result<TestClient, Box<dyn std::error::Error>> {
    let admin_state = create_test_admin_state()?;
    let auth_config = Arc::new(AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_owned(),
        api_key: Some("test-key".to_owned()),
    });

    let app_state = Arc::new(AppState {
        metrics: Arc::clone(&admin_state.metrics) as Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc::clone(&admin_state.indexing) as Arc<dyn IndexingOperationsInterface>,
        browser: Some(Arc::clone(&browse_state.browser)),
        browse_state: Some(Arc::new(browse_state)),
        mcp_server: None,
        admin_state: Some(Arc::new(admin_state)),
        auth_config: Some(auth_config),
    });
    Ok(TestClient {
        app: build_router(app_state),
    })
}

/// Helper to create metadata for a code chunk
fn create_chunk_metadata(
    file_path: &str,
    content: &str,
    start_line: u32,
    language: &str,
) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert("file_path".to_owned(), serde_json::json!(file_path));
    metadata.insert("content".to_owned(), serde_json::json!(content));
    metadata.insert("start_line".to_owned(), serde_json::json!(start_line));
    metadata.insert("language".to_owned(), serde_json::json!(language));
    metadata.insert(
        "id".to_owned(),
        serde_json::json!(format!("chunk_{}_{}", file_path, start_line)),
    );
    metadata
}

/// Helper to create a dummy embedding vector
fn create_dummy_embedding(dimensions: usize) -> Embedding {
    Embedding {
        vector: vec![0.1; dimensions],
        model: "test-model".to_owned(),
        dimensions,
    }
}

/// Populate vector store with test data simulating real indexed code
async fn populate_test_store(
    store: &dyn VectorStoreProvider,
    collection: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection_id = CollectionId::from_name(collection);
    store
        .create_collection(&collection_id, TEST_EMBEDDING_DIMENSIONS)
        .await?;

    let chunks = [
        (
            "src/lib.rs",
            "//! Main library module\n\npub mod config;\npub mod handlers;",
            1,
            "rust",
        ),
        (
            "src/lib.rs",
            "pub fn initialize() -> Result<(), Error> {\n    // Setup code\n    Ok(())\n}",
            6,
            "rust",
        ),
        (
            "src/lib.rs",
            "#[cfg(test)]\nmod tests {\n    use super::*;\n}",
            12,
            "rust",
        ),
        (
            "src/config.rs",
            "#[derive(Debug, Clone)]\npub struct Config {\n    pub host: String,\n    pub port: u16,\n}",
            1,
            "rust",
        ),
        (
            "src/config.rs",
            "impl Config {\n    pub fn new() -> Self {\n        Self { host: \"localhost\".into(), port: 8080 }\n    }\n}",
            7,
            "rust",
        ),
        (
            "src/handlers.rs",
            "use crate::config::Config;\n\npub async fn handle_request(config: &Config) -> Response {",
            1,
            "rust",
        ),
        (
            "src/handlers.rs",
            "    let result = process_data().await?;\n    Ok(Response::new(result))\n}",
            5,
            "rust",
        ),
        (
            "src/main.rs",
            "#[tokio::main]\nasync fn main() {\n    let config = Config::new();\n    println!(\"Server starting on {}:{}\", config.host, config.port);\n}",
            1,
            "rust",
        ),
    ];

    let embeddings: Vec<Embedding> = chunks
        .iter()
        .map(|_| create_dummy_embedding(TEST_EMBEDDING_DIMENSIONS))
        .collect();

    let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|(path, content, line, lang)| create_chunk_metadata(path, content, *line, lang))
        .collect();

    store
        .insert_vectors(&collection_id, &embeddings, metadata)
        .await?;
    Ok(())
}

// ============================================================================
// E2E Tests
// ============================================================================

#[tokio::test]
async fn test_e2e_real_store_list_collections() -> Result<(), Box<dyn std::error::Error>> {
    let (store, browser) = create_test_vector_store_for_e2e(TEST_EMBEDDING_DIMENSIONS)?;
    populate_test_store(store.as_ref(), "test_project").await?;

    let browse_state = BrowseState {
        browser,
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;

    let json: serde_json::Value = serde_json::from_str(&body)?;
    let expected_collection = CollectionId::from_name("test_project").to_string();

    assert!(
        body.contains(&expected_collection),
        "Should contain collection name"
    );

    let collections = json["collections"]
        .as_array()
        .ok_or("collections array missing")?;
    assert_eq!(collections.len(), 1, "Should have 1 collection");

    let collection = &collections[0];
    assert_eq!(collection["name"], expected_collection);
    assert_eq!(collection["vector_count"], 8, "Should have 8 chunks total");
    assert_eq!(collection["file_count"], 4, "Should have 4 unique files");
    assert_eq!(collection["provider"], "edgevec");
    Ok(())
}

#[tokio::test]
async fn test_e2e_real_store_list_files() -> Result<(), Box<dyn std::error::Error>> {
    let (store, browser) = create_test_vector_store_for_e2e(TEST_EMBEDDING_DIMENSIONS)?;
    populate_test_store(store.as_ref(), "test_project").await?;

    let browse_state = BrowseState {
        browser,
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections/test_project/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let files = json["files"].as_array().ok_or("files array missing")?;
    assert_eq!(files.len(), 4, "Should have 4 files");

    let file_paths: Vec<&str> = files.iter().filter_map(|f| f["path"].as_str()).collect();
    assert!(file_paths.contains(&"src/lib.rs"), "Should contain lib.rs");
    assert!(
        file_paths.contains(&"src/config.rs"),
        "Should contain config.rs"
    );
    assert!(
        file_paths.contains(&"src/handlers.rs"),
        "Should contain handlers.rs"
    );
    assert!(
        file_paths.contains(&"src/main.rs"),
        "Should contain main.rs"
    );

    for file in files {
        let path = file["path"].as_str().ok_or("path missing")?;
        let chunk_count = file["chunk_count"].as_u64().ok_or("chunk_count missing")?;
        match path {
            "src/lib.rs" => assert_eq!(chunk_count, 3, "lib.rs should have 3 chunks"),
            "src/config.rs" => assert_eq!(chunk_count, 2, "config.rs should have 2 chunks"),
            "src/handlers.rs" => assert_eq!(chunk_count, 2, "handlers.rs should have 2 chunks"),
            "src/main.rs" => assert_eq!(chunk_count, 1, "main.rs should have 1 chunk"),
            _ => return Err(format!("Unexpected file: {path}").into()),
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_e2e_real_store_get_file_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let (store, browser) = create_test_vector_store_for_e2e(TEST_EMBEDDING_DIMENSIONS)?;
    populate_test_store(store.as_ref(), "test_project").await?;

    let browse_state = BrowseState {
        browser,
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections/test_project/chunks/src/lib.rs")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let chunks = json["chunks"].as_array().ok_or("chunks array missing")?;
    assert_eq!(chunks.len(), 3, "lib.rs should have 3 chunks");

    let start_lines: Vec<u64> = chunks
        .iter()
        .filter_map(|c| c["start_line"].as_u64())
        .collect();
    assert_eq!(
        start_lines,
        vec![1, 6, 12],
        "Chunks should be sorted by line"
    );

    let first_chunk = &chunks[0];
    assert!(
        first_chunk["content"]
            .as_str()
            .ok_or("content missing")?
            .contains("Main library module"),
        "First chunk should contain module doc"
    );

    let second_chunk = &chunks[1];
    assert!(
        second_chunk["content"]
            .as_str()
            .ok_or("content missing")?
            .contains("pub fn initialize"),
        "Second chunk should contain initialize function"
    );

    let third_chunk = &chunks[2];
    assert!(
        third_chunk["content"]
            .as_str()
            .ok_or("content missing")?
            .contains("#[cfg(test)]"),
        "Third chunk should contain test module"
    );
    Ok(())
}

#[tokio::test]
async fn test_e2e_real_store_navigate_full_flow() -> Result<(), Box<dyn std::error::Error>> {
    let (store, browser) = create_test_vector_store_for_e2e(TEST_EMBEDDING_DIMENSIONS)?;
    populate_test_store(store.as_ref(), "my_rust_project").await?;

    let browse_state = BrowseState {
        browser,
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    // Step 1: List collections
    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let collections = json["collections"]
        .as_array()
        .ok_or("collections array missing")?;
    assert!(
        !collections.is_empty(),
        "Should have at least one collection"
    );

    let collection_name = collections[0]["name"]
        .as_str()
        .ok_or("collection name missing")?;
    assert_eq!(
        collection_name,
        CollectionId::from_name("my_rust_project").to_string()
    );

    // Step 2: List files in the collection
    let files_url = format!("/collections/{collection_name}/files");
    let response = client
        .get(&files_url)
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let files = json["files"].as_array().ok_or("files array missing")?;
    assert!(!files.is_empty(), "Should have files");

    let config_file = files
        .iter()
        .find(|f| f["path"].as_str() == Some("src/config.rs"))
        .ok_or("Should find config.rs")?;

    let chunk_count = config_file["chunk_count"]
        .as_u64()
        .ok_or("chunk count missing")?;
    assert_eq!(chunk_count, 2, "config.rs should have 2 chunks");

    // Step 3: Get chunks for config.rs
    let chunks_url = format!("/collections/{collection_name}/chunks/src/config.rs");
    let response = client
        .get(&chunks_url)
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let chunks = json["chunks"].as_array().ok_or("chunks array missing")?;
    assert_eq!(chunks.len(), 2, "config.rs should have 2 chunks");

    let contents: Vec<&str> = chunks
        .iter()
        .filter_map(|c| c["content"].as_str())
        .collect();

    assert!(
        contents.iter().any(|c| c.contains("pub struct Config")),
        "Should have Config struct definition"
    );
    assert!(
        contents.iter().any(|c| c.contains("impl Config")),
        "Should have Config impl block"
    );
    Ok(())
}

#[tokio::test]
async fn test_e2e_real_store_collection_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let (store, browser) = create_test_vector_store_for_e2e(TEST_EMBEDDING_DIMENSIONS)?;
    populate_test_store(store.as_ref(), "existing_collection").await?;

    let browse_state = BrowseState {
        browser,
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections/nonexistent/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert!(
        response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected 404 or 500 for non-existent collection, got {:?}",
        response.status()
    );
    Ok(())
}

#[tokio::test]
async fn test_e2e_real_store_multiple_collections() -> Result<(), Box<dyn std::error::Error>> {
    let (store, browser) = create_test_vector_store_for_e2e(TEST_EMBEDDING_DIMENSIONS)?;

    populate_test_store(store.as_ref(), "project_alpha").await?;
    populate_test_store(store.as_ref(), "project_beta").await?;

    let browse_state = BrowseState {
        browser,
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let collections = json["collections"]
        .as_array()
        .ok_or("collections array missing")?;
    assert_eq!(collections.len(), 2, "Should have 2 collections");
    let expected_alpha = CollectionId::from_name("project_alpha").to_string();
    let expected_beta = CollectionId::from_name("project_beta").to_string();

    let names: Vec<&str> = collections
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();

    assert!(
        names.contains(&expected_alpha.as_str()),
        "Should have project_alpha"
    );
    assert!(
        names.contains(&expected_beta.as_str()),
        "Should have project_beta"
    );

    assert_eq!(json["total"], 2);
    Ok(())
}
