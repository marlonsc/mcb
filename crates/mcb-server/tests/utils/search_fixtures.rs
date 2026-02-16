//! Search-specific test fixtures (unit tests only).

use mcb_domain::SearchResult;

/// Create a single test search result
pub fn create_test_search_result(
    file_path: &str,
    content: &str,
    score: f64,
    start_line: u32,
) -> SearchResult {
    SearchResult {
        id: format!("test-result-{start_line}"),
        file_path: file_path.to_owned(),
        start_line,
        content: content.to_owned(),
        score,
        language: "rust".to_owned(),
    }
}

/// Create multiple test search results
pub fn create_test_search_results(count: usize) -> Vec<SearchResult> {
    (0..count)
        .map(|i| {
            create_test_search_result(
                &format!("src/file_{i}.rs"),
                &format!("fn test_function_{i}() {{\n    // test code\n}}"),
                0.95 - (i as f64 * 0.05),
                (i as u32) * 10 + 1,
            )
        })
        .collect()
}
