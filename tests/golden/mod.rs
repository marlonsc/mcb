//! Golden acceptance tests for MCB
//!
//! These tests verify end-to-end functionality by:
//! 1. Indexing a test repository
//! 2. Performing searches
//! 3. Validating MCP tool responses match expected schemas
//! 4. Ensuring all 4 current MCP tools work correctly
//!
//! Golden tests serve as regression tests and documentation of expected behavior.

pub mod test_index_repository;
pub mod test_search_validation;
pub mod test_mcp_schemas;
pub mod test_end_to_end;

/// Test fixtures and helper utilities
pub mod fixtures {
    use std::path::PathBuf;

    /// Returns path to the test repository fixture
    pub fn test_repo_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test_repo")
    }

    /// Returns expected test collection name
    pub fn test_collection() -> &'static str {
        "mcb_golden_test"
    }
}
