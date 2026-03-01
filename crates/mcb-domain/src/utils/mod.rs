//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Cross-crate utility modules for the MCB workspace.
//!
//! **Policy**: See [`UTILITIES_POLICY.md`](./UTILITIES_POLICY.md) for naming, strictness,
//! and no-wrapper rules before adding code here.

/// Complexity and analysis utilities.
pub mod analysis;
/// Filesystem utilities.
pub mod fs;
/// ID generation, deterministic correlation (UUID v5), content hashing, and masking.
pub mod id;
/// Naming convention checks (`CamelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`).
pub mod naming;
/// Canonical path utilities — strict, no fallbacks.
pub mod path;
/// Project type detection helpers.
pub mod project_type;
/// Redaction of sensitive values in Debug/Display.
pub mod sensitivity;
/// Submodule path helpers.
pub mod submodule;
/// Canonical time utilities — strict, no fallbacks.
pub mod time;
/// VCS context data types for memory observations.
pub mod vcs_context;

#[cfg(any(test, feature = "test-utils"))]
/// Test infrastructure — fixtures, constants, service-config helpers.
pub mod tests;

// Re-export test submodules at their expected paths for backward compatibility.
#[cfg(any(test, feature = "test-utils"))]
pub use tests::assertions as test_assertions;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::collection as test_collection;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::fs_scan as test_fs_scan;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::guards as test_guards;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::search_fixtures as test_search_fixtures;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::service_detection as test_service_detection;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::services_config as test_services_config;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::sync_helpers as test_sync_helpers;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::timeouts as test_timeouts;
#[cfg(any(test, feature = "test-utils"))]
pub use tests::utils as test_utils;

pub use fs::find_files_by_extensions;
pub use id::{compute_content_hash, compute_file_hash, correlate_id, mask_id};
pub use naming::{
    get_suffix, is_camel_case, is_screaming_snake_case, is_snake_case, split_camel_case,
};
pub use sensitivity::{REDACTED, Sensitive};
