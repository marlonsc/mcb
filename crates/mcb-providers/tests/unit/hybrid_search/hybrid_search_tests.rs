//! Tests for hybrid search providers

#![cfg(feature = "hybrid-search")]

use mcb_domain::constants::search::{HYBRID_SEARCH_BM25_WEIGHT, HYBRID_SEARCH_SEMANTIC_WEIGHT};
use mcb_domain::entities::CodeChunk;
use mcb_domain::ports::HybridSearchProvider;
use mcb_domain::value_objects::SearchResult;
use mcb_providers::hybrid_search::{BM25Params, BM25Scorer, HybridSearchEngine};
use rstest::rstest;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_chunk(content: &str, file_path: &str, start_line: u32) -> CodeChunk {
    CodeChunk {
        id: format!("{file_path}:{start_line}"),
        content: content.to_owned(),
        file_path: file_path.to_owned(),
        start_line,
        end_line: start_line + content.lines().count() as u32,
        language: "Rust".to_owned(),
        metadata: serde_json::json!({}),
    }
}

fn create_test_search_result(file_path: &str, start_line: u32, score: f64) -> SearchResult {
    SearchResult {
        id: format!("{file_path}:{start_line}"),
        content: format!("Content of {file_path}:{start_line}"),
        file_path: file_path.to_owned(),
        start_line,
        score,
        language: "Rust".to_owned(),
    }
}

// ============================================================================
// BM25 Scorer Tests
// ============================================================================

#[rstest]
#[case("fn hello_world() { println!(\"Hello, World!\"); }", "hello", true)]
#[case("fn hello_world() { println!(\"Hello, World!\"); }", "world", true)]
#[case("fn hello_world() { println!(\"Hello, World!\"); }", "println", true)]
#[case("fn hello_world() { println!(\"Hello, World!\"); }", "fn", false)]
fn tokenize(#[case] input: &str, #[case] token: &str, #[case] should_contain: bool) {
    let tokens = BM25Scorer::tokenize(input);
    assert_eq!(tokens.contains(&token.to_owned()), should_contain);
}

#[rstest]
#[test]
fn test_bm25_scorer_creation() {
    let chunks = vec![
        create_test_chunk("fn authenticate_user() {}", "auth.rs", 1),
        create_test_chunk("fn validate_password() {}", "auth.rs", 10),
        create_test_chunk("fn hash_password() {}", "crypto.rs", 1),
    ];

    let scorer = BM25Scorer::new(&chunks, BM25Params::default());

    assert_eq!(scorer.total_docs(), 3);
    assert!(scorer.unique_terms() > 0);
    assert!(scorer.avg_doc_len() > 0.0);
}

#[rstest]
#[case("single")]
#[case("batch")]
fn bm25_relevant_chunk_ranks_higher(#[case] mode: &str) {
    if mode == "single" {
        let chunks = vec![
            create_test_chunk(
                "authenticate the user and validate their credentials with proper authentication",
                "auth.rs",
                1,
            ),
            create_test_chunk(
                "validate the password using hash function for security",
                "auth.rs",
                10,
            ),
            create_test_chunk(
                "process the data and compress it for storage optimization",
                "data.rs",
                1,
            ),
        ];

        let scorer = BM25Scorer::new(&chunks, BM25Params::default());
        let score_auth = scorer.score(&chunks[0], "authenticate user validate");
        let score_data = scorer.score(&chunks[2], "authenticate user validate");

        assert!(
            score_auth > score_data,
            "Auth chunk should score higher than data chunk (auth={score_auth}, data={score_data})"
        );
        return;
    }

    let chunks = vec![
        create_test_chunk(
            "search through the codebase and find matching patterns",
            "search.rs",
            1,
        ),
        create_test_chunk(
            "index the documents and build inverted index structure",
            "index.rs",
            1,
        ),
    ];

    let scorer = BM25Scorer::new(&chunks, BM25Params::default());
    let chunk_refs: Vec<&CodeChunk> = chunks.iter().collect();
    let scores = scorer.score_batch(&chunk_refs, "search codebase");

    assert_eq!(scores.len(), 2);
    assert!(
        scores[0] > scores[1],
        "First chunk should score higher (search={}, index={})",
        scores[0],
        scores[1]
    );
}

// ============================================================================
// Hybrid Search Engine Tests
// ============================================================================

#[rstest]
#[case("bm25")]
#[case("semantic")]
#[tokio::test]
async fn hybrid_search_engine_creation(#[case] weight_kind: &str) {
    let engine = HybridSearchEngine::new();
    if weight_kind == "bm25" {
        assert!((engine.bm25_weight() - HYBRID_SEARCH_BM25_WEIGHT).abs() < f64::EPSILON);
    } else {
        assert!((engine.semantic_weight() - HYBRID_SEARCH_SEMANTIC_WEIGHT).abs() < f64::EPSILON);
    }
}

#[rstest]
#[case(false, 1)]
#[case(true, 0)]
#[tokio::test]
async fn index_and_clear_collection(
    #[case] should_clear: bool,
    #[case] expected_count: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = HybridSearchEngine::new();

    let chunks = vec![
        create_test_chunk("fn authenticate_user() {}", "auth.rs", 1),
        create_test_chunk("fn validate_password() {}", "auth.rs", 10),
    ];

    engine.index_chunks("test", &chunks).await?;
    if should_clear {
        engine.clear_collection("test").await?;
    }

    let stats = engine.get_stats().await;
    assert_eq!(
        stats.get("collection_count"),
        Some(&serde_json::json!(expected_count))
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_hybrid_search() -> Result<(), Box<dyn std::error::Error>> {
    let engine = HybridSearchEngine::new();

    // Index documents with clearly distinct content
    let chunks = vec![
        create_test_chunk(
            "authenticate the user and validate their credentials for secure access",
            "auth.rs",
            1,
        ),
        create_test_chunk(
            "process the data and compress it for efficient storage optimization",
            "data.rs",
            1,
        ),
    ];
    engine.index_chunks("test", &chunks).await?;

    // Semantic results: data.rs has slightly higher semantic score
    // But auth.rs has much better BM25 match for the query
    let semantic_results = vec![
        create_test_search_result("auth.rs", 1, 0.7), // Lower semantic
        create_test_search_result("data.rs", 1, 0.75), // Higher semantic
    ];

    // Query matches auth.rs content perfectly
    let results = engine
        .search(
            "test",
            "authenticate user validate credentials",
            semantic_results,
            10,
        )
        .await?;

    assert_eq!(results.len(), 2);
    // Auth chunk should rank higher due to strong BM25 boost overcoming semantic difference
    assert_eq!(
        results[0].file_path, "auth.rs",
        "Auth should rank first due to BM25 boost"
    );
    Ok(())
}

#[rstest]
#[case(10)]
#[case(1)]
#[tokio::test]
async fn search_without_index(#[case] limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let engine = HybridSearchEngine::new();

    // Search without indexing should return semantic results as-is
    let semantic_results = vec![
        create_test_search_result("a.rs", 1, 0.9),
        create_test_search_result("b.rs", 1, 0.8),
    ];

    let results = engine
        .search("nonexistent", "query", semantic_results.clone(), limit)
        .await?;

    assert_eq!(results.len(), semantic_results.len().min(limit));
    assert_eq!(results[0].file_path, "a.rs");
    Ok(())
}
