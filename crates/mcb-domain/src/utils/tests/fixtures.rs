pub use super::utils::GOLDEN_COLLECTION;
pub use super::utils::SAMPLE_CODEBASE_FILES;

use std::path::{Path, PathBuf};

/// Path to `sample_codebase` fixture (used by golden tests).
#[must_use]
pub fn sample_codebase_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root not found")
        .join("tests/fixtures/sample_codebase")
}

/// Extract text content from `CallToolResult` for assertions (joined by space).
#[must_use]
pub fn golden_content_to_string(res: &rmcp::model::CallToolResult) -> String {
    crate::utils::text::extract_text_with_sep(&res.content, " ")
}

/// Parse "**Results found:** N" from search response text.
#[must_use]
pub fn golden_parse_results_found(text: &str) -> Option<usize> {
    let prefix = "**Results found:**";
    text.find(prefix).and_then(|i| {
        let rest = text[i + prefix.len()..].trim_start();
        let num_str: String = rest.chars().take_while(char::is_ascii_digit).collect();
        num_str.parse().ok()
    })
}

/// Count result lines (each has "📁") in search response.
#[must_use]
pub fn golden_count_result_entries(text: &str) -> usize {
    text.lines().filter(|line| line.contains("📁")).count()
}
