//! Unit test suite for mcb-infrastructure
//!
//! Run with: `cargo test -p mcb-infrastructure --test unit`
//!
//! The auth, snapshot, and sync tests require the `test-utils` feature:
//! `cargo test -p mcb-infrastructure --test unit --features test-utils`

// Shared test utilities (single declaration for all unit tests)
// We need to be careful with paths here. `main.rs` is in `tests/unit/`.
// `test_utils` folder is in `tests/test_utils/`. (Ref Step 30: `test_utils` dir in `tests/`)
// So we need `#[path = "../test_utils/mod.rs"]`?
// The original `unit.rs` was in `tests/` and had `#[path = "test_utils/mod.rs"]`.
// Now `main.rs` is in `tests/unit/`.
// So relative path is `../test_utils/mod.rs`.
#[path = "../test_utils/mod.rs"]
mod test_utils;

mod infrastructure;
mod routing;
mod services;
mod validation;

// Loose files in tests/unit/
mod config_figment_tests;
mod constants_tests;
mod crypto_tests;
mod di_tests;
mod error_ext_tests;
mod events_tests;
mod file_hash_tests;
mod fts_check_tests;
mod health_tests;
mod logging_tests;
mod mcp_context_config_tests;

mod service_tests;
