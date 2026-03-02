pub use super::utils::GOLDEN_COLLECTION;
pub use super::utils::SAMPLE_CODEBASE_FILES;

pub use super::mcp_assertions::{
    golden_content_to_string, golden_count_result_entries, golden_parse_results_found,
};

use std::path::{Path, PathBuf};

/// Path to `sample_codebase` fixture (used by golden tests).
///
/// # Panics
///
/// Panics if the workspace root cannot be resolved from `CARGO_MANIFEST_DIR`.
#[must_use]
pub fn sample_codebase_path() -> PathBuf {
    #[allow(clippy::expect_used)]
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root not found")
        .join("tests/fixtures/sample_codebase")
}
