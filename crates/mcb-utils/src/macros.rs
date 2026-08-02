//! Crate-internal macros for mcb-utils.
//!
//! Centralized macro definitions used across the crate.

/// Batch-define `pub const` string constants with optional doc comments.
///
/// Used by sub-modules to reduce boilerplate in constant definitions.
macro_rules! define_str_consts {
    ($($(#[doc = $doc:literal])? $name:ident = $val:literal;)*) => {
        $($(#[doc = $doc])? pub const $name: &str = $val;)*
    };
}
