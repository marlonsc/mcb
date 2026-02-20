//! Tests for search domain services — using shared `AppContext` for performance.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mcb_application::use_cases::SearchServiceImpl;
use mcb_domain::entities::CodeChunk;
use mcb_domain::ports::{ContextServiceInterface, SearchServiceInterface};
use mcb_domain::utils::id;
use mcb_domain::value_objects::CollectionId;
use rstest::*;
use serde_json::json;

use crate::utils::shared_context::try_shared_app_context;

static COLLECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_collection(prefix: &str) -> CollectionId {
    let n = COLLECTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    CollectionId::from_uuid(id::deterministic("collection", &format!("{prefix}_{n}")))
}

#[fixture]
async fn ctx() -> Option<Arc<dyn ContextServiceInterface>> {
    let Some(app_ctx) = try_shared_app_context() else {
        return None;
    };
    let services = app_ctx
        .build_domain_services()
        .await
        .expect("build domain services");
    Some(services.context_service)
}

#[fixture]
fn test_chunks() -> Vec<CodeChunk> {
    vec![
        CodeChunk {
            id: "config_chunk".to_owned(),
            file_path: "src/config.rs".to_owned(),
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
            .to_owned(),
            start_line: 1,
            end_line: 15,
            language: "rust".to_owned(),
            metadata: json!({"type": "struct", "name": "Config"}),
        },
        CodeChunk {
            id: "auth_chunk".to_owned(),
            file_path: "src/auth.rs".to_owned(),
            content: "pub async fn authenticate(token: &str) -> Result<User, AuthError> {
    let claims = verify_jwt(token)?;
    let user = User::from_claims(claims);
    Ok(user)
}
"
            .to_owned(),
            start_line: 1,
            end_line: 10,
            language: "rust".to_owned(),
            metadata: json!({"type": "function", "name": "authenticate"}),
        },
        CodeChunk {
            id: "handler_chunk".to_owned(),
            file_path: "src/handlers.rs".to_owned(),
            content: "pub async fn handle_request(req: Request) -> Response {
    let config = Config::new();
    let result = process_data(&req, &config).await?;
    Response::ok(result)
}"
            .to_owned(),
            start_line: 1,
            end_line: 5,
            language: "rust".to_owned(),
            metadata: json!({"type": "function", "name": "handle_request"}),
        },
    ]
}

#[rstest]
#[tokio::test]
async fn test_search_service_creation(#[future] ctx: Option<Arc<dyn ContextServiceInterface>>) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let search_service = SearchServiceImpl::new(svc);
    let _service: Box<dyn SearchServiceInterface> = Box::new(search_service);
}

#[rstest]
#[tokio::test]
async fn test_search_service_indexing_flow(
    #[future] ctx: Option<Arc<dyn ContextServiceInterface>>,
    test_chunks: Vec<CodeChunk>,
) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    let col_id = unique_collection("indexing_flow");
    svc.initialize(&col_id).await.expect("init failed");

    svc.store_chunks(&col_id, &test_chunks)
        .await
        .expect("store failed");

    let search_service = SearchServiceImpl::new(svc);
    let results = search_service
        .search(&col_id, "configuration settings", 10)
        .await
        .expect("search failed");

    assert!(!results.is_empty(), "Should find results");
}

#[rstest]
#[tokio::test]
async fn test_search_empty_collection(#[future] ctx: Option<Arc<dyn ContextServiceInterface>>) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let col_id = unique_collection("empty");
    svc.initialize(&col_id).await.expect("init failed");

    let search_service = SearchServiceImpl::new(svc);
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
async fn test_context_service_capabilities(
    #[future] ctx: Option<Arc<dyn ContextServiceInterface>>,
) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    assert_eq!(svc.embedding_dimensions(), 384);

    let embedding = svc.embed_text("test query").await.expect("embed failed");
    assert_eq!(embedding.dimensions, 384);
    assert_eq!(embedding.vector.len(), 384);
    assert!(!embedding.model.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_store_and_retrieve_chunks(
    #[future] ctx: Option<Arc<dyn ContextServiceInterface>>,
    test_chunks: Vec<CodeChunk>,
) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let col_id = unique_collection("store");

    svc.initialize(&col_id).await.expect("init failed");
    svc.store_chunks(&col_id, &test_chunks)
        .await
        .expect("store failed");

    let results = svc
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
async fn test_clear_collection(
    #[future] ctx: Option<Arc<dyn ContextServiceInterface>>,
    test_chunks: Vec<CodeChunk>,
) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let col_id = unique_collection("clear");

    svc.initialize(&col_id).await.expect("init");
    svc.store_chunks(&col_id, &test_chunks)
        .await
        .expect("store");

    let results = svc
        .search_similar(&col_id, "config", 5)
        .await
        .expect("search");
    assert!(!results.is_empty());

    svc.clear_collection(&col_id).await.expect("clear");

    if let Ok(results) = svc.search_similar(&col_id, "config", 5).await {
        assert!(results.is_empty());
    }
}

#[rstest]
#[case("/tmp/absolute.rs", "fn absolute() {}", true)]
#[case("./src/normalized.rs", "fn normalized() {}", false)]
#[tokio::test]
async fn test_path_handling(
    #[future] ctx: Option<Arc<dyn ContextServiceInterface>>,
    #[case] file_path: &str,
    #[case] content: &str,
    #[case] should_fail: bool,
) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let col_id = unique_collection("path");
    svc.initialize(&col_id).await.expect("init");

    let chunk = CodeChunk {
        id: "chunk_1".to_owned(),
        file_path: file_path.to_owned(),
        content: content.to_owned(),
        start_line: 1,
        end_line: 1,
        language: "rust".to_owned(),
        metadata: json!({}),
    };

    let res = svc.store_chunks(&col_id, &[chunk]).await;

    if should_fail {
        assert!(res.is_err(), "Should reject absolute path");
    } else {
        res.expect("Should accept relative path");
    }
}

#[rstest]
#[tokio::test]
async fn test_full_search_flow(
    #[future] ctx: Option<Arc<dyn ContextServiceInterface>>,
    test_chunks: Vec<CodeChunk>,
) {
    let Some(svc) = ctx.await else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let col_id = unique_collection("full_search");
    svc.initialize(&col_id).await.expect("init");
    svc.store_chunks(&col_id, &test_chunks)
        .await
        .expect("store");

    let search_service = SearchServiceImpl::new(Arc::clone(&svc));
    let results = search_service
        .search(&col_id, "request handler", 5)
        .await
        .expect("search");

    assert!(
        !results.is_empty(),
        "search for 'request handler' should return results from indexed chunks"
    );
}
