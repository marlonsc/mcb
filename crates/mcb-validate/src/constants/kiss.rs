//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Constants for KISS validators (complexity, nesting, param count).

/// Type name suffixes that identify DI container structs (allowed more fields).
pub const DI_CONTAINER_SUFFIXES: &[&str] = &["Context", "Container", "Components", "State"];

/// Type name substrings that identify config-like structs (allowed more fields).
pub const DI_CONTAINER_CONTAINS: &[&str] = &["Config", "Settings"];

/// Minimum line distance between reported nesting violations to avoid noise.
pub const NESTING_PROXIMITY_THRESHOLD: usize = 5;

/// Self parameter variants to exclude from function parameter counting.
pub const SELF_PARAM_VARIANTS: &[&str] = &["&self", "self", "&mut self"];
