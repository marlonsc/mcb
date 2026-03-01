//! Test infrastructure — centralized utilities and service configuration.
//!
//! **Documentation**: [docs/modules/domain.md#testing-utilities](../../../../docs/modules/domain.md#testing-utilities)
//!
//! Submodules:
//! - [`utils`] — fixtures, constants, helpers, `require_service!` macro
//! - [`services_config`] — `config/tests.toml` lookup for external service endpoints
//! - [`guards`] — RAII guards for env vars, current dir, file backup
//! - [`fs_scan`] — filesystem scanning helpers for architecture tests
//! - [`assertions`] — generic test assertion helpers
//! - [`service_detection`] — external service availability checks, CI detection
//! - [`sync_helpers`] — `Arc<Mutex<T>>` helper constructors
//! - [`collection`] — unique collection name generator
//! - [`timeouts`] — test timeout constants and polling helpers
//! - [`search_fixtures`] — `SearchResult` test fixture builders

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
