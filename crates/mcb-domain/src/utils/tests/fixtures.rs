use std::path::{Path, PathBuf};

/// Path to the `sample_codebase` fixture (used by golden tests).
///
/// This is resolved relative to `CARGO_MANIFEST_DIR` for the `mcb-domain` crate.
#[must_use]
pub fn sample_codebase_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/utils/tests/fixtures/sample_codebase")
}
