//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Pattern Registry Module
//!
//! Provides centralized regex pattern management loaded from YAML rules.
//! Patterns are compiled once at startup and accessed via a global registry.

mod helpers;
mod registry;

pub use registry::{PATTERNS, PatternRegistry, default_rules_dir};

pub(crate) use helpers::{required_pattern, required_patterns};
