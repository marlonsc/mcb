//! Integration tests for browse API endpoints
//!
//! Tests the REST API for browsing indexed collections, files, and chunks.

use async_trait::async_trait;
use mcb_application::ports::admin::{
    IndexingOperation, IndexingOperationsInterface, PerformanceMetricsData,
    PerformanceMetricsInterface,
};
use mcb_application::ports::infrastructure::events::{DomainEventStream, EventBusProvider};
use mcb_domain::error::Result;
use mcb_domain::events::DomainEvent;
use mcb_domain::ports::providers::VectorStoreBrowser;
use mcb_domain::value_objects::{CollectionInfo, FileInfo, SearchResult};
use mcb_server::admin::auth::AdminAuthConfig;
use mcb_server::admin::browse_handlers::BrowseState;
use mcb_server::admin::handlers::AdminState;
use mcb_server::admin::routes::admin_rocket;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock VectorStoreBrowser for testing
pub struct MockVectorStoreBrowser {
    collections: Vec<CollectionInfo>,
    files: Vec<FileInfo>,
    chunks: Vec<SearchResult>,
}

impl MockVectorStoreBrowser {
    pub fn new() -> Self {
        Self {
            collections: Vec::new(),
            files: Vec::new(),
            chunks: Vec::new(),
        }
    }

    pub fn with_collections(mut self, collections: Vec<CollectionInfo>) -> Self {
        self.collections = collections;
        self
    }

    pub fn with_files(mut self, files: Vec<FileInfo>) -> Self {
        self.files = files;
        self
    }

    pub fn with_chunks(mut self, chunks: Vec<SearchResult>) -> Self {
        self.chunks = chunks;
        self
    }
}

#[async_trait]
impl VectorStoreBrowser for MockVectorStoreBrowser {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        Ok(self.collections.clone())
    }

    async fn list_file_paths(&self, _collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        Ok(self.files.iter().take(limit).cloned().collect())
    }

    async fn get_chunks_by_file(
        &self,
        _collection: &str,
        _file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        Ok(self.chunks.clone())
    }
}

// ============================================================================
// Mock Implementations
// ============================================================================

/// Mock performance metrics
struct MockMetrics;

impl PerformanceMetricsInterface for MockMetrics {
    fn uptime_secs(&self) -> u64 {
        0
    }

    fn record_query(&self, _response_time_ms: u64, _success: bool, _cache_hit: bool) {}

    fn update_active_connections(&self, _delta: i64) {}

    fn get_performance_metrics(&self) -> PerformanceMetricsData {
        PerformanceMetricsData {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            active_connections: 0,
            uptime_seconds: 0,
        }
    }
}

/// Mock indexing operations
struct MockIndexing;

impl IndexingOperationsInterface for MockIndexing {
    fn get_operations(&self) -> HashMap<String, IndexingOperation> {
        HashMap::new()
    }
}

/// Mock event bus
struct MockEventBus;

#[async_trait]
impl EventBusProvider for MockEventBus {
    async fn publish_event(&self, _event: DomainEvent) -> Result<()> {
        Ok(())
    }

    async fn subscribe_events(&self) -> Result<DomainEventStream> {
        // Return an empty stream
        Ok(Box::pin(futures::stream::empty()))
    }

    fn has_subscribers(&self) -> bool {
        false
    }

    async fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn subscribe(&self, _topic: &str) -> Result<String> {
        Ok("mock-subscription".to_string())
    }
}

/// Create test admin state with minimal dependencies
fn create_test_admin_state() -> AdminState {
    AdminState {
        metrics: Arc::new(MockMetrics),
        indexing: Arc::new(MockIndexing),
        config_watcher: None,
        config_path: None,
        shutdown_coordinator: None,
        shutdown_timeout_secs: 30,
        event_bus: Arc::new(MockEventBus),
        service_manager: None,
        cache: None,
    }
}

/// Create a test Rocket client with browse state
async fn create_test_client(browse_state: BrowseState) -> Client {
    let admin_state = create_test_admin_state();
    let auth_config = Arc::new(AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_string(),
        api_key: Some("test-key".to_string()),
    });

    let rocket = admin_rocket(admin_state, auth_config, Some(browse_state));
    Client::tracked(rocket)
        .await
        .expect("valid rocket instance")
}

#[tokio::test]
async fn test_list_collections_empty() {
    let browser = MockVectorStoreBrowser::new();
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("\"collections\":[]"));
    assert!(body.contains("\"total\":0"));
}

#[tokio::test]
async fn test_list_collections_with_data() {
    let collections = vec![
        CollectionInfo::new("test_collection".to_string(), 100, 10, None, "memory"),
        CollectionInfo::new("another_collection".to_string(), 50, 5, None, "memory"),
    ];

    let browser = MockVectorStoreBrowser::new().with_collections(collections);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("test_collection"));
    assert!(body.contains("another_collection"));
    assert!(body.contains("\"total\":2"));
}

#[tokio::test]
async fn test_list_files_in_collection() {
    let files = vec![
        FileInfo::new("src/main.rs".to_string(), 5, "rust", None),
        FileInfo::new("src/lib.rs".to_string(), 3, "rust", None),
    ];

    let browser = MockVectorStoreBrowser::new().with_files(files);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections/test_collection/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("src/main.rs"));
    assert!(body.contains("src/lib.rs"));
    assert!(body.contains("\"total\":2"));
}

#[tokio::test]
async fn test_get_file_chunks() {
    let chunks = vec![
        SearchResult {
            id: "chunk_1".to_string(),
            file_path: "src/main.rs".to_string(),
            content: "fn main() { }".to_string(),
            start_line: 1,
            score: 1.0,
            language: "rust".to_string(),
        },
        SearchResult {
            id: "chunk_2".to_string(),
            file_path: "src/main.rs".to_string(),
            content: "fn helper() { }".to_string(),
            start_line: 5,
            score: 1.0,
            language: "rust".to_string(),
        },
    ];

    let browser = MockVectorStoreBrowser::new().with_chunks(chunks);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    let response = client
        .get("/collections/test_collection/chunks/src/main.rs")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.expect("response body");
    assert!(body.contains("fn main()"));
    assert!(body.contains("fn helper()"));
    assert!(body.contains("\"total\":2"));
}

#[tokio::test]
async fn test_browse_requires_auth() {
    let browser = MockVectorStoreBrowser::new();
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    // Request without auth header
    let response = client.get("/collections").dispatch().await;

    // Should return unauthorized (401) or forbidden (403)
    assert!(
        response.status() == Status::Unauthorized || response.status() == Status::Forbidden,
        "Expected 401 or 403, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_browse_invalid_auth() {
    let browser = MockVectorStoreBrowser::new();
    let browse_state = BrowseState {
        browser: Arc::new(browser),
    };

    let client = create_test_client(browse_state).await;

    // Request with invalid auth key
    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "invalid-key"))
        .dispatch()
        .await;

    // Should return unauthorized (401) or forbidden (403)
    assert!(
        response.status() == Status::Unauthorized || response.status() == Status::Forbidden,
        "Expected 401 or 403, got {:?}",
        response.status()
    );
}
