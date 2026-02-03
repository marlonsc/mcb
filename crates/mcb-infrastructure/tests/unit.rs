//! Unit test suite for mcb-infrastructure
//!
//! Run with: `cargo test -p mcb-infrastructure --test unit`
//!
//! The auth, snapshot, and sync tests require the `test-utils` feature:
//! `cargo test -p mcb-infrastructure --test unit --features test-utils`

#[path = "unit/constants_tests.rs"]
mod constants_tests;

#[path = "unit/crypto_tests.rs"]
mod crypto_tests;

#[path = "unit/error_ext_tests.rs"]
mod error_ext_tests;

#[path = "unit/health_tests.rs"]
mod health_tests;

#[path = "unit/logging_tests.rs"]
mod logging_tests;

#[path = "unit/di_tests.rs"]
mod di_tests;

#[path = "unit/router_tests.rs"]
mod router_tests;

#[path = "unit/lifecycle_tests.rs"]
mod lifecycle_tests;

#[path = "unit/config_figment_tests.rs"]
mod config_figment_tests;

#[path = "unit/prometheus_metrics_tests.rs"]
mod prometheus_metrics_tests;

// Validation service tests - require validation feature
#[cfg(feature = "validation")]
#[path = "unit/validation_service_tests.rs"]
mod validation_service_tests;

// Infrastructure service tests (require test-utils feature)
#[cfg(feature = "test-utils")]
#[path = "unit/auth_tests.rs"]
mod auth_tests;

#[cfg(feature = "test-utils")]
#[path = "unit/snapshot_tests.rs"]
mod snapshot_tests;

#[cfg(feature = "test-utils")]
#[path = "unit/sync_tests.rs"]
mod sync_tests;

#[path = "unit/file_hash_tests.rs"]
mod file_hash_tests;

#[path = "unit/fts_check_tests.rs"]
mod fts_check_tests;

#[path = "unit/memory_fts_tests.rs"]
mod memory_fts_tests;

#[path = "unit/memory_repository_tests.rs"]
mod memory_repository_tests;
