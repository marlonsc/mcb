//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Naming validation helper macros.
//!
//! - `apply_ca_rule!`: Clean Architecture naming rule application (used by `naming/checks/ca.rs`).

/// Applies a CA naming rule: returns `Some(violation)` when the file matches the matcher
/// and is not in any of the required directories.
#[macro_export]
macro_rules! apply_ca_rule {
    ($path:expr, $file_name:expr, $path_str:expr, $matcher:expr, $required_dirs:expr, $detected_type:expr, $issue:expr, $suggestion:expr, $severity:expr) => {
        (name_matches($file_name, $matcher) && !in_any_dir($path_str, $required_dirs))
            .then(|| ca_violation($path, $detected_type, $issue, $suggestion, $severity))
    };
}
