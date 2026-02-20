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
/// Submodule path helpers.
pub mod submodule;
/// Canonical time utilities — strict, no fallbacks.
pub mod time;
/// VCS context data types for memory observations.
pub mod vcs_context;

pub use fs::find_files_by_extensions;
pub use id::{compute_content_hash, compute_file_hash, correlate_id, mask_id};
pub use naming::{
    get_suffix, is_camel_case, is_screaming_snake_case, is_snake_case, split_camel_case,
};
