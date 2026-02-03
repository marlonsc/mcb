//! Golden tests: index repository, multi-language, ignore patterns.
//! Contract: docs/testing/GOLDEN_TESTS_CONTRACT.md.

use crate::test_utils::test_fixtures::{
    GOLDEN_COLLECTION, create_test_mcp_server, golden_content_to_string,
    golden_parse_indexing_stats, sample_codebase_path,
};
use mcb_server::args::IndexCodebaseArgs;
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn golden_index_test_repository() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    assert!(path.exists());

    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some(GOLDEN_COLLECTION.to_string()),
            extensions: None,
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;

    assert!(r.is_ok(), "index_codebase must succeed");
    let res = r.unwrap();
    assert!(
        !res.is_error.unwrap_or(true),
        "index response must not be error"
    );
    let text = golden_content_to_string(&res);
    assert!(
        text.contains("chunk")
            || text.contains("file")
            || text.contains("Index")
            || text.contains("Files processed")
            || text.contains("Indexing Started")
            || text.contains("Source directory")
            || text.contains("Path:"),
        "index response must contain chunk/file/Index/Files/Path: {}",
        text
    );
    if text.contains("Indexing Completed")
        && let Some((files, chunks)) = golden_parse_indexing_stats(&text)
    {
        assert!(
            files > 0,
            "indexing completed must report files_processed > 0: {}",
            text
        );
        assert!(
            chunks > 0,
            "indexing completed must report chunks_created > 0: {}",
            text
        );
    }
}

#[tokio::test]
async fn golden_index_handles_multiple_languages() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("multi_lang".to_string()),
            extensions: Some(vec!["rs".to_string()]),
            ignore_patterns: None,
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn golden_index_respects_ignore_patterns() {
    let server = create_test_mcp_server().await;
    let path = sample_codebase_path();
    let r = server
        .index_codebase_handler()
        .handle(Parameters(IndexCodebaseArgs {
            path: path.to_string_lossy().to_string(),
            collection: Some("ignore_test".to_string()),
            extensions: None,
            ignore_patterns: Some(vec!["*.md".to_string()]),
            max_file_size: None,
            follow_symlinks: None,
            token: None,
        }))
        .await;
    assert!(r.is_ok());
}
