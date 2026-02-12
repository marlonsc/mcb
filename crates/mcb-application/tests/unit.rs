//! Unit test suite for mcb-application
//!
//! Run with: `cargo test -p mcb-application --test unit`

#[path = "unit/search_tests.rs"]
mod search_tests;

#[path = "unit/use_cases_tests.rs"]
mod use_cases_tests;

#[path = "unit/registry_tests.rs"]
mod registry_tests;

#[path = "unit/instrumented_embedding_tests.rs"]
mod instrumented_embedding_tests;

// vcs_indexing module was removed in v0.2.2; tests are stale
// #[path = "unit/vcs_indexing_tests.rs"]
// mod vcs_indexing_tests;

#[path = "unit/memory_service_tests.rs"]
mod memory_service_tests;

#[path = "unit/memory_tests.rs"]
mod memory_tests;

#[path = "unit/test_utils.rs"]
pub mod test_utils;
