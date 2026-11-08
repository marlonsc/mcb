use std::path::{Path, PathBuf};

/// Path to the `sample_codebase` fixture (used by golden tests).
///
/// This is resolved relative to `CARGO_MANIFEST_DIR` for the `mcb-domain` crate.
#[must_use]
pub fn sample_codebase_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/utils/tests/fixtures/sample_codebase")
}

/// Path to the canonical `golden_queries.json` fixture (used by golden tests).
///
/// Single source of truth; downstream crates must not vendor their own copy.
#[must_use]
pub fn golden_queries_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/utils/tests/fixtures/golden_queries.json")
}
