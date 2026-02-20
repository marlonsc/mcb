//! Integration tests for browse API endpoints
//!
//! Tests the REST API for browsing indexed collections, files, and chunks.
//! Uses mock `VectorStoreBrowser` for isolation.
//!
//! E2E tests with real `EdgeVec` stores live in `tests/e2e/browse_e2e.rs`.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::Result as DomainResult;
use mcb_domain::ports::HighlightServiceInterface;
use mcb_domain::ports::VectorStoreBrowser;
use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};
use mcb_infrastructure::infrastructure::{
    AtomicPerformanceMetrics, DefaultIndexingOperations, default_event_bus,
};
use mcb_infrastructure::services::highlight_service::HighlightServiceImpl;

use mcb_server::admin::auth::AdminAuthConfig;
use mcb_server::admin::browse_handlers::BrowseState;
use mcb_server::admin::handlers::AdminState;
use mcb_server::admin::routes::admin_rocket;
use rocket::http::{Header, Status};
use rocket::local::asynchronous::Client;
use rstest::rstest;

/// Mock `VectorStoreBrowser` for testing
pub struct TestVectorStoreBrowser {
    collections: Vec<CollectionInfo>,
    files: Vec<FileInfo>,
    chunks: Vec<SearchResult>,
}

impl TestVectorStoreBrowser {
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
impl VectorStoreBrowser for TestVectorStoreBrowser {
    async fn list_collections(&self) -> DomainResult<Vec<CollectionInfo>> {
        Ok(self.collections.clone())
    }

    async fn list_file_paths(
        &self,
        _collection: &CollectionId,
        limit: usize,
    ) -> DomainResult<Vec<FileInfo>> {
        Ok(self.files.iter().take(limit).cloned().collect())
    }

    async fn get_chunks_by_file(
        &self,
        _collection: &CollectionId,
        _file_path: &str,
    ) -> DomainResult<Vec<SearchResult>> {
        Ok(self.chunks.clone())
    }
}

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

/// Create a test Rocket client with browse state
async fn create_test_client(
    browse_state: BrowseState,
) -> Result<Client, Box<dyn std::error::Error>> {
    let admin_state = create_test_admin_state()?;
    let auth_config = Arc::new(AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_owned(),
        api_key: Some("test-key".to_owned()),
    });

    let rocket = admin_rocket(admin_state, auth_config, Some(browse_state));
    Ok(Client::tracked(rocket).await?)
}

fn create_test_browse_state(browser: TestVectorStoreBrowser) -> BrowseState {
    BrowseState {
        browser: Arc::new(browser),
        highlight_service: create_test_highlight_service(),
    }
}

#[rstest]
#[case(vec![], 0, None, None)]
#[case(
    vec![
        CollectionInfo::new("test_collection", 100, 10, None, "memory"),
        CollectionInfo::new("another_collection", 50, 5, None, "memory"),
    ],
    2,
    Some("test_collection"),
    Some("another_collection")
)]
#[tokio::test]
async fn test_list_collections(
    #[case] collections: Vec<CollectionInfo>,
    #[case] expected_total: usize,
    #[case] expected_name_a: Option<&str>,
    #[case] expected_name_b: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let browser = TestVectorStoreBrowser::new().with_collections(collections);
    let browse_state = create_test_browse_state(browser);
    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    assert!(body.contains(&format!("\"total\":{expected_total}")));
    if expected_total == 0 {
        assert!(body.contains("\"collections\":[]"));
        return Ok(());
    }
    if let Some(name) = expected_name_a {
        assert!(body.contains(name));
    }
    if let Some(name) = expected_name_b {
        assert!(body.contains(name));
    }
    Ok(())
}

#[tokio::test]
async fn test_list_files_in_collection() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        FileInfo::new("src/main.rs".to_owned(), 5, "rust", None),
        FileInfo::new("src/lib.rs".to_owned(), 3, "rust", None),
    ];

    let browser = TestVectorStoreBrowser::new().with_files(files);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections/test_collection/files")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    assert!(body.contains("src/main.rs"));
    assert!(body.contains("src/lib.rs"));
    assert!(body.contains("\"total\":2"));
    Ok(())
}

#[tokio::test]
async fn test_get_file_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let chunks = vec![
        SearchResult {
            id: "chunk_1".to_owned(),
            file_path: "src/main.rs".to_owned(),
            content: "fn main() { }".to_owned(),
            start_line: 1,
            score: 1.0,
            language: "rust".to_owned(),
        },
        SearchResult {
            id: "chunk_2".to_owned(),
            file_path: "src/main.rs".to_owned(),
            content: "fn helper() { }".to_owned(),
            start_line: 5,
            score: 1.0,
            language: "rust".to_owned(),
        },
    ];

    let browser = TestVectorStoreBrowser::new().with_chunks(chunks);
    let browse_state = BrowseState {
        browser: Arc::new(browser),
        highlight_service: create_test_highlight_service(),
    };

    let client = create_test_client(browse_state).await?;

    let response = client
        .get("/collections/test_collection/chunks/src/main.rs")
        .header(Header::new("X-Admin-Key", "test-key"))
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    let body = response
        .into_string()
        .await
        .ok_or("response body missing")?;
    assert!(body.contains("fn main()"));
    assert!(body.contains("fn helper()"));
    assert!(body.contains("\"total\":2"));
    Ok(())
}

#[rstest]
#[case(None)]
#[case(Some("invalid-key".to_owned()))]
#[tokio::test]
async fn test_browse_auth_validation(
    #[case] admin_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let browse_state = create_test_browse_state(TestVectorStoreBrowser::new());
    let client = create_test_client(browse_state).await?;

    let mut request = client.get("/collections");
    if let Some(key) = admin_key {
        request = request.header(Header::new("X-Admin-Key", key));
    }
    let response = request.dispatch().await;

    assert!(
        response.status() == Status::Unauthorized || response.status() == Status::Forbidden,
        "Expected 401 or 403, got {:?}",
        response.status()
    );
    Ok(())
}
