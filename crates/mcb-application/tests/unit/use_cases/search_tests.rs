//! Tests for search domain services
//!
//! These tests use real providers (FastEmbedProvider, MokaCacheProvider, EdgeVecVectorStoreProvider)
//! to validate actual search behavior, not mocked responses.

use rstest::rstest;
use std::sync::Arc;

use mcb_application::use_cases::SearchServiceImpl;
use mcb_domain::entities::CodeChunk;
use mcb_domain::ports::services::*;
use mcb_domain::utils::id;
use mcb_domain::value_objects::CollectionId;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use rstest::*;
use serde_json::json;
use tempfile::TempDir;

async fn create_real_context_service() -> (Arc<dyn ContextServiceInterface>, TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));

    let ctx = init_app(config).await.expect("init app context");
    let services = ctx
        .build_domain_services()
        .await
        .expect("build domain services");

    (services.context_service, temp_dir)
}

struct TestContext {
    service: Arc<dyn ContextServiceInterface>,
    _temp: TempDir,
}

#[fixture]
async fn ctx() -> TestContext {
    let (service, _temp) = create_real_context_service().await;
    TestContext { service, _temp }
}

#[fixture]
fn test_chunks() -> Vec<CodeChunk> {
    vec![
        CodeChunk {
            id: "config_chunk".to_string(),
            file_path: "src/config.rs".to_string(),
            content: r#"#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            database_url: "postgres://localhost/db".to_string(),
        }
    }
}"#
            .to_string(),
            start_line: 1,
            end_line: 15,
            language: "rust".to_string(),
            metadata: json!({"type": "struct", "name": "Config"}),
        },
        CodeChunk {
            id: "auth_chunk".to_string(),
            file_path: "src/auth.rs".to_string(),
            content: r#"pub async fn authenticate(token: &str) -> Result<User, AuthError> {
    let claims = verify_jwt(token)?;
    let user = User::from_claims(claims);
    Ok(user)
}
"#
            .to_string(),
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "authenticate"}),
        },
        CodeChunk {
            id: "handler_chunk".to_string(),
            file_path: "src/handlers.rs".to_string(),
            content: r#"pub async fn handle_request(req: Request) -> Response {
    let config = Config::new();
    let result = process_data(&req, &config).await?;
    Response::ok(result)
}"#
            .to_string(),
            start_line: 1,
            end_line: 5,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "handle_request"}),
        },
    ]
}

#[rstest]
#[tokio::test]
async fn test_search_service_creation(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let search_service = SearchServiceImpl::new(ctx.service);
    let _service: Box<dyn SearchServiceInterface> = Box::new(search_service);
}

#[rstest]
#[tokio::test]
async fn test_search_service_indexing_flow(
    #[future] ctx: TestContext,
    test_chunks: Vec<CodeChunk>,
) {
    let ctx = ctx.await;

    // Initialize
    let col_id = CollectionId::from_uuid(id::deterministic("collection", "test_collection"));
    ctx.service.initialize(&col_id).await.expect("init failed");

    // Store
    ctx.service
        .store_chunks(&col_id, &test_chunks)
        .await
        .expect("store failed");

    // Search via SearchService
    let search_service = SearchServiceImpl::new(ctx.service);
    let results = search_service
        .search(&col_id, "configuration settings", 10)
        .await
        .expect("search failed");

    assert!(!results.is_empty(), "Should find results");
}

#[rstest]
#[tokio::test]
async fn test_search_empty_collection(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let col_id = CollectionId::from_uuid(id::deterministic("collection", "empty_collection"));
    ctx.service.initialize(&col_id).await.expect("init failed");

    let search_service = SearchServiceImpl::new(ctx.service);
    let results = search_service
        .search(&col_id, "anything", 10)
        .await
        .expect("search failed");

    assert!(
        results.is_empty(),
        "Empty collection should yield empty results"
    );
}

#[rstest]
#[tokio::test]
async fn test_context_service_capabilities(#[future] ctx: TestContext) {
    let ctx = ctx.await;

    // Dimensions
    assert_eq!(ctx.service.embedding_dimensions(), 384);

    // Embed text
    let embedding = ctx
        .service
        .embed_text("test query")
        .await
        .expect("embed failed");
    assert_eq!(embedding.dimensions, 384);
    assert_eq!(embedding.vector.len(), 384);
    assert!(!embedding.model.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_store_and_retrieve_chunks(#[future] ctx: TestContext, test_chunks: Vec<CodeChunk>) {
    let ctx = ctx.await;
    let col_id = CollectionId::from_uuid(id::deterministic("collection", "store_test"));

    ctx.service.initialize(&col_id).await.expect("init failed");
    ctx.service
        .store_chunks(&col_id, &test_chunks)
        .await
        .expect("store failed");

    let results = ctx
        .service
        .search_similar(&col_id, "authenticate user token", 5)
        .await
        .expect("search failed");

    assert!(!results.is_empty());
    let first = &results[0];
    assert!(!first.file_path.is_empty());
    assert!(!first.content.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_clear_collection(#[future] ctx: TestContext, test_chunks: Vec<CodeChunk>) {
    let ctx = ctx.await;
    let col_id = CollectionId::from_uuid(id::deterministic("collection", "clear_test"));

    ctx.service.initialize(&col_id).await.expect("init");
    ctx.service
        .store_chunks(&col_id, &test_chunks)
        .await
        .expect("store");

    // Verify data exists
    let results = ctx
        .service
        .search_similar(&col_id, "config", 5)
        .await
        .expect("search");
    assert!(!results.is_empty());

    // Clear
    ctx.service.clear_collection(&col_id).await.expect("clear");

    // Verify empty
    if let Ok(results) = ctx.service.search_similar(&col_id, "config", 5).await {
        assert!(results.is_empty());
    }
}

#[rstest]
#[case("/tmp/absolute.rs", "fn absolute() {}", true)]
#[case("./src/normalized.rs", "fn normalized() {}", false)]
#[tokio::test]
async fn test_path_handling(
    #[future] ctx: TestContext,
    #[case] file_path: &str,
    #[case] content: &str,
    #[case] should_fail: bool,
) {
    let ctx = ctx.await;
    let col_id = CollectionId::from_uuid(id::deterministic("collection", "path_test"));
    ctx.service.initialize(&col_id).await.expect("init");

    let chunk = CodeChunk {
        id: "chunk_1".to_string(),
        file_path: file_path.to_string(),
        content: content.to_string(),
        start_line: 1,
        end_line: 1,
        language: "rust".to_string(),
        metadata: json!({}),
    };

    let res = ctx.service.store_chunks(&col_id, &[chunk]).await;

    if should_fail {
        assert!(res.is_err(), "Should reject absolute path");
    } else {
        res.expect("Should accept relative path");
        // Verify normalization if needed (searching for normalized path)
        let results = ctx
            .service
            .search_similar(&col_id, "normalized", 1)
            .await
            .expect("search");
        if file_path.starts_with("./") {
            // Expect normalization to remove ./
            let expected = file_path.trim_start_matches("./");
            assert!(results.iter().any(|r| r.file_path == expected));
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_full_search_flow(#[future] ctx: TestContext, test_chunks: Vec<CodeChunk>) {
    let ctx = ctx.await;
    let col_id = CollectionId::from_uuid(id::deterministic("collection", "architecture_test"));
    ctx.service.initialize(&col_id).await.expect("init");
    ctx.service
        .store_chunks(&col_id, &test_chunks)
        .await
        .expect("store");

    let search_service = SearchServiceImpl::new(ctx.service.clone());
    let results = search_service
        .search(&col_id, "request handler", 5)
        .await
        .expect("search");

    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .any(|r| r.content.contains("handle") || r.file_path.contains("handler"))
    );
}
