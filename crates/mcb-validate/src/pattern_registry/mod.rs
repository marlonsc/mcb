//! Pattern Registry Module
//!
//! Provides centralized regex pattern management loaded from YAML rules.
//! Patterns are compiled once at startup and accessed via a global registry.

mod registry;

pub use registry::{PATTERNS, PatternRegistry, default_rules_dir};
