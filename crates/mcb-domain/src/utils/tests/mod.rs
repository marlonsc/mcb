//! Test infrastructure — centralized utilities and service configuration.
//!
//! This module serves as the **Centralized Test Scaffolding** for the entire project.
//! It provides:
//! - **DI-Ready Fixtures**: Reusable builders for domain entities (Users, Projects, Sessions).
//! - **Centralized Constants**: Stable IDs and timestamps for consistent multi-tenant testing.
//! - **Invariant Assertions**: Shared logic for verifying MCP tool result shapes and JSON-RPC compliance.
//! - **Environment Guards**: RAII handlers for safe environment variable and filesystem manipulation in tests.
//!
//! **Architecture Pattern**: Instead of each crate defining its own test helpers,
//! they MUST import from this module (re-exported via `mcb_domain::utils::tests`)
//! to ensure that logic changes in the domain layer are automatically reflected across the test suite.
//!
//! **Documentation**: [docs/modules/domain.md#testing-utilities](../../../../docs/modules/domain.md#testing-utilities)
//!
//! ### Submodules:
//! - [`utils`] — Primary fixtures, constants, and the `require_service!` macro.
//! - [`mcp_assertions`] — Canonical assertions for `CallToolResult` and text extraction.
//! - [`json_helpers`] — Shared JSON parsing and validation for test outputs.
//! - [`registry`] — (Internal) Registry of test-friendly provider factories.

/// Centralized test fixtures, constants, helpers, and `require_service!` macro.
pub mod utils;

/// Test-only configuration helpers for external service endpoints.
pub mod services_config;

/// RAII guards for test isolation (env vars, cwd, file backup).
pub mod guards;

/// Filesystem scanning helpers (scan_rs_files, rust_files_under).
pub mod fs_scan;

/// Generic test assertion helpers (assert_no_violations, assert_violations_exact, etc.).
pub mod assertions;

/// External service detection and skip macros.
pub mod service_detection;

/// Thread-safe wrapper helpers (`arc_mutex`, `arc_mutex_vec`, etc.).
pub mod sync_helpers;

/// Unique collection name generator for test isolation.
pub mod collection;

/// Test timeout constants and polling helpers.
pub mod timeouts;

/// `SearchResult` test fixture builders.
pub mod search_fixtures;

/// JSON parsing and extraction helpers for test assertions.
pub mod json_helpers;

/// MCP `CallToolResult` assertion helpers (error_text, is_error, assert_error_shape, etc.).
pub mod mcp_assertions;

/// `CodeChunk` test fixture builders.
pub mod chunk_fixtures;

/// Git test repository helpers (create_test_repo, run_git).
pub mod git_helpers;
