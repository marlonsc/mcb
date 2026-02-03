//! Unit test suite for mcb-providers
//!
//! Run with: `cargo test -p mcb-providers --test unit --features hybrid-search`

#[cfg(feature = "hybrid-search")]
#[path = "unit/hybrid_search_tests.rs"]
mod hybrid_search_tests;

#[path = "unit/submodule_tests.rs"]
mod submodule_tests;

#[path = "unit/git2_provider_tests.rs"]
mod git2_provider_tests;

#[path = "unit/cargo_tests.rs"]
mod cargo_tests;

#[path = "unit/go_tests.rs"]
mod go_tests;

#[path = "unit/maven_tests.rs"]
mod maven_tests;

#[path = "unit/npm_tests.rs"]
mod npm_tests;

#[path = "unit/python_tests.rs"]
mod python_tests;
