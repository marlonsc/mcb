//! Integration Tests for mcb-infrastructure
//!
//! This module provides shared test utilities for integration tests.
//!
//! Test Structure:
//! - `tests/unit.rs` - Unit tests (constants, crypto, `error_ext`, health, logging)
//! - `tests/integration.rs` - Integration tests (cache, config, di, utils)
//! - `tests/utils/` - Real provider factories for full-stack testing
//!
//! Run all tests: `cargo test -p mcb-infrastructure`
//! Run unit tests: `cargo test -p mcb-infrastructure --test unit`
//! Run integration: `cargo test -p mcb-infrastructure --test integration`

// Shared AppContext for test performance (avoids repeated ONNX model loads)
mod shared_context;

// Real provider test utilities for full-stack integration testing
pub mod utils;

// Shared test utilities
/// Shared test helper functions.
pub mod test_helpers {
    /// Create a temporary test directory
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created.
    pub fn temp_dir() -> std::io::Result<tempfile::TempDir> {
        tempfile::tempdir()
    }
}
